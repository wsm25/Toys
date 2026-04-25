fn main() {
    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("benches/plf_colony_bench.cpp")
        .compile("plf_colony_bench");

    println!("cargo:rerun-if-changed=benches/plf_colony_bench.cpp");
}
