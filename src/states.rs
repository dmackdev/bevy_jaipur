#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InitGame,
    TurnTransition,
    InGame,
    WaitForTweensToFinish,
    GameOver,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TurnState {
    None,
    Take,
    Sell,
}
