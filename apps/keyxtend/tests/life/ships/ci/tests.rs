//! The reader over small workflow snippets, one per way a step can build the app with its defaults off.

use super::lists;

/// The package the snippets build.
const APP: &str = "keyxtend";

/// The app's feature lists in `yaml`.
fn app(yaml: &str) -> Vec<Vec<&str>> {
    lists(yaml, APP)
}

#[test]
fn a_folded_step_is_read_across_its_lines() {
    let yaml = r"
      - run: >-
          cargo clippy -p keyxtend --all-targets --no-default-features
          --features b,a --locked -- -D warnings -F c
      - run: cargo xtask tidy
";
    assert_eq!(app(yaml), [["a", "b"]]);
}

#[test]
fn long_flags_count_in_any_order() {
    let yaml = r"
      - run: cargo clippy --features=b,a --locked --package keyxtend --no-default-features
      - run: cargo test --no-default-features --package=keyxtend --features 'c'
";
    assert_eq!(app(yaml), [vec!["a", "b"], vec!["c"]]);
}

#[test]
fn the_short_feature_flag_counts() {
    let yaml = "      - run: cargo clippy -F a,b -p keyxtend   --no-default-features\n";
    assert_eq!(app(yaml), [["a", "b"]]);
}

#[test]
fn a_named_block_step_reads_each_cargo_command() {
    let yaml = r"
      - name: Lint
        run: |
          cargo fmt --all --check
          cargo clippy -p keyxtend --no-default-features --features a
        env:
          RUSTFLAGS: --features b
";
    assert_eq!(app(yaml), [["a"]]);
}

#[test]
fn a_commented_out_step_does_not_count() {
    let yaml = r"
      # - run: >-
      #     cargo clippy -p keyxtend --all-targets --no-default-features --features a
      - run: cargo clippy -p keyxtend --features gate-input
";
    assert!(app(yaml).is_empty());
}

#[test]
fn other_packages_and_default_builds_do_not_count() {
    let yaml = r"
      - run: cargo clippy -p kx-ui --no-default-features --features a
      - run: cargo test -p keyxtend --features a
";
    assert!(app(yaml).is_empty());
}
