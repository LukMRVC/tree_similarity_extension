create or replace procedure tree_ds_query()
LANGUAGE plpgsql AS $$
DECLARE
    t timestamptz := clock_timestamp();
    c_query CURSOR FOR
    SELECT
        threshold,
        query_tree
    FROM
        tree_select_ds LIMIT 50;

    output_count INT;
BEGIN
    FOR rowvar in c_query LOOP
        SELECT COUNT(*) INTO output_count
        FROM tree_ds
        WHERE tree_lb_bounded_label_intersect(rowvar.query_tree, tree, rowvar.threshold) <= rowvar.threshold;

        raise notice 'Matched trees %', output_count;
    END LOOP;
raise notice 'time spent=%', clock_timestamp() - t;

END; $$;
