use wendao_nexus_runtime::sha256_content_hash;

#[test]
fn content_hash_is_stable_and_namespaced() {
    assert_eq!(
        sha256_content_hash(b"wendao"),
        "sha256:432eb228043388bbca0214a08e28ab04519315e0c5f717a09de63fd8a1741c67"
    );
}
