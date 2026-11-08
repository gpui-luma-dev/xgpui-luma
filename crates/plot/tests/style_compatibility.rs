use xgpui_luma_plot::{
    CartesianChartBuilder, CartesianStyle, CartesianStyleProvider, Dataset, PlotStyle, PlotStyleProvider, presentation,
};

struct LegacyProvider;

impl PlotStyleProvider for LegacyProvider {
    fn resolve(&self) -> PlotStyle {
        presentation::PlotStyle::default()
    }
}

#[test]
fn legacy_style_and_provider_names_share_the_cartesian_contract() {
    fn resolve(provider: impl CartesianStyleProvider) -> CartesianStyle {
        provider.resolve()
    }

    let legacy: PlotStyle = resolve(LegacyProvider);
    let canonical: CartesianStyle = legacy;
    let provider: &dyn presentation::PlotStyleProvider = &canonical;
    let restored: presentation::CartesianStyle = provider.resolve();
    assert_eq!(restored.line.stroke.color, canonical.line.stroke.color);

    let _chart = CartesianChartBuilder::new(Dataset::new()).style(LegacyProvider).build();
}
