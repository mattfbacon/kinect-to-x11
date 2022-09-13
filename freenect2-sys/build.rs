fn main() {
	println!("cargo:rustc-link-lib=freenect2");
	println!("cargo:rerun-if-changed=src/wrapper.hpp");
	println!("cargo:rerun-if-changed=src/wrapper.cpp");

	cc::Build::new()
		.file("src/wrapper.cpp")
		.cpp(true)
		.include("src")
		.compile("wrapper");

	let bindings = bindgen::builder()
		.header("src/wrapper.hpp")
		.parse_callbacks(Box::new(bindgen::CargoCallbacks))
		.allowlist_function("fn2_.*")
		.allowlist_type("Fn2.*")
		.size_t_is_usize(true)
		.generate()
		.expect("failed to generate bindings");

	let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
	bindings
		.write_to_file(out_path.join("bindings.rs"))
		.expect("failed to write bindings");
}
