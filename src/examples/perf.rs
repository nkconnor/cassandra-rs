//~ #include <assert.h>
//~ #include <string.h>
//~ #include <stdio.h>
//~ #include <stdlib.h>
//~ #include <math.h>
//~ #include <uv.h>
//~ #include "cassandra.h"
//~ /*
//~ * Use this example with caution. It's just used as a scratch example for debugging and
//~ * roughly analyzing performance.
//~ */
//~ #define NUM_THREADS 1
//~ #define NUM_IO_WORKER_THREADS 4
//~ #define NUM_CONCURRENT_REQUESTS 10000
//~ #define NUM_SAMPLES 1000
//~ #define USE_PREPARED 1
//~ const char* big_string = "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567"
//~ "0123456701234567012345670123456701234567012345670123456701234567";
//~ UuidGen* uuid_gen;
//~ typedef struct ThreadStats_ {
//~ long count;
//~ double total_averages;
//~ double samples[NUM_SAMPLES];
//~ } ThreadStats;
//~ void print_error(Future* future) {
//~ CassString message = cass_future_error_message(future);
//~ fprintf(stderr, "Error: %.*s\n", (int)message.length, message.data);
//~ }
//~ Cluster* create_cluster() {
//~ Cluster* cluster = cass_cluster_new();
//~ cass_cluster_set_contact_points(cluster, "127.0.0.1");
//~ cass_cluster_set_credentials(cluster, "cassandra", "cassandra");
//~ cass_cluster_set_num_threads_io(cluster, NUM_IO_WORKER_THREADS);
//~ cass_cluster_set_queue_size_io(cluster, 10000);
//~ cass_cluster_set_pending_requests_low_water_mark(cluster, 5000);
//~ cass_cluster_set_pending_requests_high_water_mark(cluster, 10000);
//~ cass_cluster_set_core_connections_per_host(cluster, 1);
//~ cass_cluster_set_max_connections_per_host(cluster, 2);
//~ return cluster;
//~ }
//~ CassandraError connect_session(Session* session, const Cluster* cluster) {
//~ CassandraError rc = CASS_OK;
//~ Future* future = cass_session_connect_keyspace(session, cluster, "examples");
//~ cass_future_wait(future);
//~ rc = cass_future_error_code(future);
//~ if (rc != CASS_OK) {
//~ print_error(future);
//~ }
//~ cass_future_free(future);
//~ return rc;
//~ }
//~ CassandraError execute_query(Session* session, const char* query) {
//~ CassandraError rc = CASS_OK;
//~ Future* future = NULL;
//~ Statement* statement = cass_statement_new(cass_string_init(query), 0);
//~ future = cass_session_execute(session, statement);
//~ cass_future_wait(future);
//~ rc = cass_future_error_code(future);
//~ if (rc != CASS_OK) {
//~ print_error(future);
//~ }
//~ cass_future_free(future);
//~ cass_statement_free(statement);
//~ return rc;
//~ }
//~ CassandraError prepare_query(Session* session, CassString query, const PreparedStatement** prepared) {
//~ CassandraError rc = CASS_OK;
//~ Future* future = NULL;
//~ future = cass_session_prepare(session, query);
//~ cass_future_wait(future);
//~ rc = cass_future_error_code(future);
//~ if (rc != CASS_OK) {
//~ print_error(future);
//~ } else {
//~ *prepared = cass_future_get_prepared(future);
//~ }
//~ cass_future_free(future);
//~ return rc;
//~ }
//~ int compare_dbl(const void* d1, const void* d2) {
//~ if (*((double*)d1) < *((double*)d2)) {
//~ return -1;
//~ } else if (*((double*)d1) > *((double*)d2)) {
//~ return 1;
//~ } else {
//~ return 0;
//~ }
//~ }
//~ void print_thread_stats(ThreadStats* thread_stats) {
//~ double throughput_avg = 0.0;
//~ double throughput_min = 0.0;
//~ double throughput_median = 0.0;
//~ double throughput_max = 0.0;
//~ int index_median = ceil(0.5 * NUM_SAMPLES);
//~ qsort(thread_stats->samples, NUM_SAMPLES, sizeof(double), compare_dbl);
//~ throughput_avg = thread_stats->total_averages / thread_stats->count;
//~ throughput_min = thread_stats->samples[0];
//~ throughput_median = thread_stats->samples[index_median];
//~ throughput_max = thread_stats->samples[NUM_SAMPLES - 1];
//~ printf("%d IO threads, %d requests/batch:\navg: %f\nmin: %f\nmedian: %f\nmax: %f\n",
//~ NUM_IO_WORKER_THREADS,
//~ NUM_CONCURRENT_REQUESTS,
//~ throughput_avg,
//~ throughput_min,
//~ throughput_median,
//~ throughput_max);
//~ }
//~ void insert_into_perf(Session* session, CassString query, const PreparedStatement* prepared,
//~ ThreadStats* thread_stats) {
//~ int i;
//~ double elapsed, throughput;
//~ uint64_t start;
//~ int num_requests = 0;
//~ Future* futures[NUM_CONCURRENT_REQUESTS];
//~ unsigned long thread_id = uv_thread_self();
//~ CassCollection* collection = cass_collection_new(CASS_COLLECTION_TYPE_SET, 2);
//~ cass_collection_append_string(collection, cass_string_init("jazz"));
//~ cass_collection_append_string(collection, cass_string_init("2013"));
//~ start = uv_hrtime();
//~ for (i = 0; i < NUM_CONCURRENT_REQUESTS; ++i) {
//~ Uuid id;
//~ Statement* statement;
//~ if (prepared != NULL) {
//~ statement = cass_prepared_bind(prepared);
//~ } else {
//~ statement = cass_statement_new(query, 5);
//~ }
//~ cass_uuid_gen_time(uuid_gen, &id);
//~ cass_statement_bind_uuid(statement, 0, id);
//~ cass_statement_bind_string(statement, 1, cass_string_init(big_string));
//~ cass_statement_bind_string(statement, 2, cass_string_init(big_string));
//~ cass_statement_bind_string(statement, 3, cass_string_init(big_string));
//~ cass_statement_bind_collection(statement, 4, collection);
//~ futures[i] = cass_session_execute(session, statement);
//~ cass_statement_free(statement);
//~ }
//~ for (i = 0; i < NUM_CONCURRENT_REQUESTS; ++i) {
//~ Future* future = futures[i];
//~ CassandraError rc = cass_future_error_code(future);
//~ if (rc != CASS_OK) {
//~ print_error(future);
//~ } else {
//~ num_requests++;
//~ }
//~ cass_future_free(future);
//~ }
//~ elapsed = (double)(uv_hrtime() - start) / 1000000000.0;
//~ throughput = (double)num_requests / elapsed;
//~ thread_stats->samples[thread_stats->count++] = throughput;
//~ thread_stats->total_averages += throughput;
//~ printf("%ld: average %f inserts/sec (%d, %f)\n", thread_id, thread_stats->total_averages / thread_stats->count, num_requests, elapsed);
//~ cass_collection_free(collection);
//~ }
//~ void run_insert_queries(void* data) {
//~ int i;
//~ Session* session = (Session*)data;
//~ const PreparedStatement* insert_prepared = NULL;
//~ CassString insert_query = cass_string_init("INSERT INTO songs (id, title, album, artist, tags) VALUES (?, ?, ?, ?, ?);");
//~ ThreadStats thread_stats;
//~ thread_stats.count = 0;
//~ thread_stats.total_averages = 0.0;
//~ #if USE_PREPARED
//~ if (prepare_query(session, insert_query, &insert_prepared) == CASS_OK) {
//~ #endif
//~ for (i = 0; i < NUM_SAMPLES; ++i) {
//~ insert_into_perf(session, insert_query, insert_prepared, &thread_stats);
//~ }
//~ #if USE_PREPARED
//~ cass_prepared_free(insert_prepared);
//~ }
//~ #endif
//~ print_thread_stats(&thread_stats);
//~ }
//~ void select_from_perf(Session* session, CassString query, const PreparedStatement* prepared,
//~ ThreadStats* thread_stats) {
//~ int i;
//~ double elapsed, throughput;
//~ uint64_t start;
//~ int num_requests = 0;
//~ Future* futures[NUM_CONCURRENT_REQUESTS];
//~ unsigned long thread_id = uv_thread_self();
//~ start = uv_hrtime();
//~ for (i = 0; i < NUM_CONCURRENT_REQUESTS; ++i) {
//~ Statement* statement;
//~ if (prepared != NULL) {
//~ statement = cass_prepared_bind(prepared);
//~ } else {
//~ statement = cass_statement_new(query, 0);
//~ }
//~ futures[i] = cass_session_execute(session, statement);
//~ cass_statement_free(statement);
//~ }
//~ for (i = 0; i < NUM_CONCURRENT_REQUESTS; ++i) {
//~ Future* future = futures[i];
//~ CassandraError rc = cass_future_error_code(future);
//~ if (rc != CASS_OK) {
//~ print_error(future);
//~ } else {
//~ const CassandraResult* result = cass_future_get_result(future);
//~ assert(cass_result_column_count(result) == 6);
//~ cass_result_free(result);
//~ num_requests++;
//~ }
//~ cass_future_free(future);
//~ }
//~ elapsed = (double)(uv_hrtime() - start) / 1000000000.0;
//~ throughput = (double)num_requests / elapsed;
//~ thread_stats->samples[thread_stats->count++] = throughput;
//~ thread_stats->total_averages += throughput;
//~ printf("%ld: average %f selects/sec (%d, %f)\n", thread_id, thread_stats->total_averages / thread_stats->count, num_requests, elapsed);
//~ }
//~ void run_select_queries(void* data) {
//~ int i;
//~ Session* session = (Session*)data;
//~ const PreparedStatement* select_prepared = NULL;
//~ CassString select_query = cass_string_init("SELECT * FROM songs WHERE id = a98d21b2-1900-11e4-b97b-e5e358e71e0d");
//~ ThreadStats thread_stats;
//~ thread_stats.count = 0;
//~ thread_stats.total_averages = 0.0;
//~ #if USE_PREPARED
//~ if (prepare_query(session, select_query, &select_prepared) == CASS_OK) {
//~ #endif
//~ for (i = 0; i < NUM_SAMPLES; ++i) {
//~ select_from_perf(session, select_query, select_prepared, &thread_stats);
//~ }
//~ #if USE_PREPARED
//~ cass_prepared_free(select_prepared);
//~ }
//~ #endif
//~ print_thread_stats(&thread_stats);
//~ }
//~ int main() {
//~ int i;
//~ uv_thread_t threads[NUM_THREADS];
//~ Cluster* cluster = NULL;
//~ Session* session = NULL;
//~ Future* close_future = NULL;
//~ cass_log_set_level(CASS_LOG_INFO);
//~ cluster = create_cluster();
//~ uuid_gen = cass_uuid_gen_new();
//~ session = cass_session_new();
//~ if (connect_session(session, cluster) != CASS_OK) {
//~ cass_cluster_free(cluster);
//~ cass_session_free(session);
//~ return -1;
//~ }
//~ execute_query(session,
//~ "INSERT INTO songs (id, title, album, artist, tags) VALUES "
//~ "(a98d21b2-1900-11e4-b97b-e5e358e71e0d, "
//~ "'La Petite Tonkinoise', 'Bye Bye Blackbird', 'Joséphine Baker', { 'jazz', '2013' });");
//~ #define DO_SELECTS
//~ for (i = 0; i < NUM_THREADS; ++i) {
//~ #ifdef DO_INSERTS
//~ uv_thread_create(&threads[i], run_insert_queries, (void*)session);
//~ #endif
//~ #ifdef DO_SELECTS
//~ uv_thread_create(&threads[i], run_select_queries, (void*)session);
//~ #endif
//~ }
//~ for (i = 0; i < NUM_THREADS; ++i) {
//~ uv_thread_join(&threads[i]);
//~ }
//~ close_future = cass_session_close(session);
//~ cass_future_wait(close_future);
//~ cass_future_free(close_future);
//~ cass_cluster_free(cluster);
//~ cass_uuid_gen_free(uuid_gen);
//~ return 0;
//~ }
