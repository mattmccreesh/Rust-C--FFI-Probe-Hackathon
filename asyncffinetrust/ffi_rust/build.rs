fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("ffi_rust")
        .generate_csharp_file("dotnet/NativeMethods.g.cs")
        .unwrap();
}