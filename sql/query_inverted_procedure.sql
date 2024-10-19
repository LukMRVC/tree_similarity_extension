create or replace procedure inverted_ds_query()
LANGUAGE plpgsql
AS $$
DECLARE
        t timestamptz := clock_timestamp();

    c_query CURSOR FOR
    SELECT
        id,
        threshold,
        query_tree
    FROM
        inverted_select_ds
      order by id;

    output_count INT;
    total_output_cnt INT DEFAULT 0;
BEGIN
    FOR rowvar in c_query LOOP
        SELECT COUNT(*) INTO output_count
        FROM inverted_ds
        WHERE inverted_bounded_tree_label_intersect(rowvar.query_tree, tree, rowvar.threshold) <= rowvar.threshold;
        total_output_cnt := total_output_cnt + output_count;
        raise notice 'Query % matched trees %', rowvar.id, output_count;
    END LOOP;

    raise notice 'time spent=%', clock_timestamp() - t;
    raise notice 'Total trees selected=%', total_output_cnt;
END; $$;




/*
explain (analyze, buffers, verbose) select * from 
inverted_select_ds id
join inverted_ds ds on inverted_bounded_tree_label_intersect(id.query_tree, ds.tree, id.threshold) <= id.threshold

SELECT 
  c.relname, 
  pg_relation_size(c.oid) AS table_size_bytes,
  count(*) * 8192 AS cache_size_bytes,  -- assuming 8KB block size
  (count(*) * 8192)::float / pg_relation_size(c.oid) * 100 AS cache_percent
FROM 
  pg_buffercache b
JOIN 
  pg_class c ON b.relfilenode = pg_relation_filenode(c.oid) 
WHERE 
  c.relname = 'inverted_ds'
GROUP BY 
  c.relname, c.oid;

*/