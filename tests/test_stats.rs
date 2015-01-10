use std::borrow::ToOwned;
use std::io::process;

use workdir::Workdir;

macro_rules! stats_tests {
    ($name:ident, $field:expr, $rows:expr, $expect:expr) => (
        stats_tests!($name, $field, $rows, $expect, false);
    );
    ($name:ident, $field:expr, $rows:expr, $expect:expr, $nulls:expr) => (
        mod $name {
            use super::test_stats;

            stats_test_headers!($name, $field, $rows, $expect, $nulls);
            stats_test_no_headers!($name, $field, $rows, $expect, $nulls);
        }
    );
}

macro_rules! stats_test_headers {
    ($name:ident, $field:expr, $rows:expr, $expect:expr) => (
        stats_test_headers!($name, $field, $rows, $expect, false);
    );
    ($name:ident, $field:expr, $rows:expr, $expect:expr, $nulls:expr) => (
        #[test]
        fn headers_no_index() {
            let name = concat!(stringify!($name), "_headers_no_index");
            test_stats(name, $field, $rows, $expect, true, false, $nulls);
        }

        #[test]
        fn headers_index() {
            let name = concat!(stringify!($name), "_headers_index");
            test_stats(name, $field, $rows, $expect, true, true, $nulls);
        }
    );
}

macro_rules! stats_test_no_headers {
    ($name:ident, $field:expr, $rows:expr, $expect:expr) => (
        stats_test_no_headers!($name, $field, $rows, $expect, false);
    );
    ($name:ident, $field:expr, $rows:expr, $expect:expr, $nulls:expr) => (
        #[test]
        fn no_headers_no_index() {
            let name = concat!(stringify!($name), "_no_headers_no_index");
            test_stats(name, $field, $rows, $expect, false, false, $nulls);
        }

        #[test]
        fn no_headers_index() {
            let name = concat!(stringify!($name), "_no_headers_index");
            test_stats(name, $field, $rows, $expect, false, true, $nulls);
        }
    );
}

fn test_stats<S: Str>(name: S, field: &str, rows: &[&str], expected: &str,
                      headers: bool, use_index: bool, nulls: bool) {
    let (wrk, mut cmd) = setup(name, rows, headers, use_index, nulls);
    let field_val = get_field_value(&wrk, &mut cmd, field);
    assert_eq!(field_val, expected.to_owned());
}

fn setup<S: Str>(name: S, rows: &[&str],
                 headers: bool, use_index: bool, nulls: bool)
                -> (Workdir, process::Command) {
    let wrk = Workdir::new(name.as_slice());
    let mut data: Vec<Vec<String>> =
        rows.iter().map(|&s| vec![s.to_owned()]).collect();
    if headers { data.insert(0, svec!["header"]); }
    if use_index {
        wrk.create_indexed("in.csv", data);
    } else {
        wrk.create("in.csv", data);
    }

    let mut cmd = wrk.command("stats");
    cmd.arg("in.csv");
    if !headers { cmd.arg("--no-headers"); }
    if nulls { cmd.arg("--nulls"); }

    (wrk, cmd)
}

fn get_field_value(wrk: &Workdir, cmd: &mut process::Command, field: &str)
                  -> String {
    if field == "median" { cmd.arg("--median"); }
    if field == "cardinality" { cmd.arg("--cardinality"); }
    if field == "mode" { cmd.arg("--mode"); }

    let mut rows: Vec<Vec<String>> = wrk.read_stdout(cmd);
    let headers = rows.remove(0);
    for row in rows.iter() {
        for (h, val) in headers.iter().zip(row.iter()) {
            if h.as_slice() == field {
                return val.clone();
            }
        }
    }
    panic!("BUG: Could not find field '{}' in headers '{:?}' \
            for command '{}'.", field, headers, cmd);
}

stats_tests!(stats_infer_unicode, "type", &["a"], "Unicode");
stats_tests!(stats_infer_int, "type", &["1"], "Integer");
stats_tests!(stats_infer_float, "type", &["1.2"], "Float");
stats_tests!(stats_infer_null, "type", &[""], "NULL");
stats_tests!(stats_infer_unicode_null, "type", &["a", ""], "Unicode");
stats_tests!(stats_infer_int_null, "type", &["1", ""], "Integer");
stats_tests!(stats_infer_float_null, "type", &["1.2", ""], "Float");
stats_tests!(stats_infer_null_unicode, "type", &["", "a"], "Unicode");
stats_tests!(stats_infer_null_int, "type", &["", "1"], "Integer");
stats_tests!(stats_infer_null_float, "type", &["", "1.2"], "Float");
stats_tests!(stats_infer_int_unicode, "type", &["1", "a"], "Unicode");
stats_tests!(stats_infer_unicode_int, "type", &["a", "1"], "Unicode");
stats_tests!(stats_infer_int_float, "type", &["1", "1.2"], "Float");
stats_tests!(stats_infer_float_int, "type", &["1.2", "1"], "Float");
stats_tests!(stats_infer_null_int_float_unicode, "type",
             &["", "1", "1.2", "a"], "Unicode");

stats_tests!(stats_no_mean, "mean", &["a"], "");
stats_tests!(stats_no_stddev, "stddev", &["a"], "");
stats_tests!(stats_no_median, "median", &["a"], "");
stats_tests!(stats_no_mode, "mode", &["a", "b"], "N/A");

stats_tests!(stats_null_mean, "mean", &[""], "");
stats_tests!(stats_null_stddev, "stddev", &[""], "");
stats_tests!(stats_null_median, "median", &[""], "");
stats_tests!(stats_null_mode, "mode", &[""], "N/A");

stats_tests!(stats_includenulls_null_mean, "mean", &[""], "", true);
stats_tests!(stats_includenulls_null_stddev, "stddev", &[""], "", true);
stats_tests!(stats_includenulls_null_median, "median", &[""], "", true);
stats_tests!(stats_includenulls_null_mode, "mode", &[""], "N/A", true);

stats_tests!(stats_includenulls_mean,
             "mean", &["5", "", "15", "10"], "7.5", true);

stats_tests!(stats_min, "min", &["2", "1.1"], "1.1");
stats_tests!(stats_max, "max", &["2", "1.1"], "2");
stats_tests!(stats_min_mix, "min", &["2", "a", "1.1"], "1.1");
stats_tests!(stats_max_mix, "max", &["2", "a", "1.1"], "a");
stats_tests!(stats_min_null, "min", &["", "2", "1.1"], "1.1");
stats_tests!(stats_max_null, "max", &["2", "1.1", ""], "2");

stats_tests!(stats_len_min, "min_length", &["aa", "a"], "1");
stats_tests!(stats_len_max, "max_length", &["a", "aa"], "2");
stats_tests!(stats_len_min_null, "min_length", &["", "aa", "a"], "0");
stats_tests!(stats_len_max_null, "max_length", &["a", "aa", ""], "2");

stats_tests!(stats_mean, "mean", &["5", "15", "10"], "10");
stats_tests!(stats_stddev, "stddev", &["1", "2", "3"], "0.816497");
stats_tests!(stats_mean_null, "mean", &["", "5", "15", "10"], "10");
stats_tests!(stats_stddev_null, "stddev", &["1", "2", "3", ""], "0.816497");
stats_tests!(stats_mean_mix, "mean", &["5", "15.1", "9.9"], "10");
stats_tests!(stats_stddev_mix, "stddev", &["1", "2.1", "2.9"], "0.778888");

stats_tests!(stats_cardinality, "cardinality", &["a", "b", "a"], "2");
stats_tests!(stats_mode, "mode", &["a", "b", "a"], "a");
stats_tests!(stats_mode_null, "mode", &["", "a", "b", "a"], "a");
stats_tests!(stats_median, "median", &["1", "2", "3"], "2");
stats_tests!(stats_median_null, "median", &["", "1", "2", "3"], "2");
stats_tests!(stats_median_even, "median", &["1", "2", "3", "4"], "2.5");
stats_tests!(stats_median_even_null, "median",
             &["", "1", "2", "3", "4"], "2.5");
stats_tests!(stats_median_mix, "median", &["1", "2.5", "3"], "2.5");

mod stats_infer_nothing {
    // Only test CSV data with headers.
    // Empty CSV data with no headers won't produce any statistical analysis.
    use super::test_stats;
    stats_test_headers!(stats_infer_nothing, "type", &[], "NULL");
}

mod stats_zero_cardinality {
    use super::test_stats;
    stats_test_headers!(stats_zero_cardinality, "cardinality", &[], "0");
}

mod stats_zero_mode {
    use super::test_stats;
    stats_test_headers!(stats_zero_mode, "mode", &[], "N/A");
}

mod stats_zero_mean {
    use super::test_stats;
    stats_test_headers!(stats_zero_mean, "mean", &[], "");
}

mod stats_zero_median {
    use super::test_stats;
    stats_test_headers!(stats_zero_median, "median", &[], "");
}
