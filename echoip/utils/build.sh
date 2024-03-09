OUTPUT=target/output
mkdir -p $OUTPUT
for arch in "x86_64" "i686" "aarch64"; do
    for libc in "gnu" "musl"; do
        config_target=${arch}_unknown_linux_${libc}
        rust_target=${arch}-unknown-linux-${libc}
        echo building on $rust_target
        bash -c "CARGO_TARGET_${config_target^^}_LINKER=${arch}-linux-gnu-gcc \
            CC_${config_target}=${arch}-linux-gnu-gcc \
            CXX_${config_target}=${arch}-linux-gnu-g++ \
            cargo build -r --target ${rust_target}"
        for exe in $(find target/${rust_target}/release/ -maxdepth 1 -type f -executable); do
            cp $exe $OUTPUT/${rust_target}-$(basename $exe)
        done
    done
done