#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InitGame,
    TurnTransition,
    InGame,
}
