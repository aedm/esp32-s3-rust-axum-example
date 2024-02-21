fn main() -> anyhow::Result<()> {
    build_data::set_GIT_BRANCH();
    build_data::set_GIT_COMMIT();
    build_data::set_SOURCE_TIMESTAMP();
    build_data::set_RUSTC_VERSION();
    build_data::no_debug_rebuilds();

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;

    Ok(())
}
