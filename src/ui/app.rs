use eframe::{egui::{self}, CreationContext};

use crate::{analyzer::board_interactions::AnalyzerController, database::create::create_database, engine::{board::BoardMetaData, Board}, game::{controller::GameController, evaluator::Evaluator}, ui::{theme, ui_setting::{UiController, UiSettings}, DEFAULT_FEN}};

pub enum HistoryScreenVariant { PastGameSelectionView, GameAnalyzerView(BoardMetaData)}
pub enum AppScreen {
    MainMenu,
    TrainWithAi,
    Multiplayer,
    History(HistoryScreenVariant),
    Analyze,
}
#[derive(Clone)]
pub enum PopupType { GameLostPopup(String), SavingGamePopup }
pub struct MyApp {
    pub screen: AppScreen,
    pub popup: Option<PopupType>,
    pub theme: theme::ThemeLoader,
    pub board: Board,
    pub game: GameController,
    pub evaluator: Evaluator,
    pub past_games: Option<Vec<BoardMetaData>>,
    pub ui_settings: UiSettings,
    pub ui_controller: UiController,
    pub analyzer: AnalyzerController
}




impl From<&CreationContext<'_>> for MyApp{

    fn from(cc : &CreationContext) -> Self {

        let mut app = 
            Self {
                screen: AppScreen::MainMenu,
                popup: None,
                theme: theme::ThemeLoader::from(cc),
                board: Board::default(),
                game: GameController::default(),
                evaluator: Evaluator::new(),
                past_games: None,
                ui_settings: UiSettings::default(),
                ui_controller: UiController::default(),
                analyzer: AnalyzerController::default(),
                

            };
        app.start_evaluator();
        let res = create_database();
        println!("{:?}", res);
        return app

        

    }

}
//Update loop
impl eframe::App for MyApp {

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Clone the popup before the match statement to avoid multiple mutable borrows
        let popup_clone = self.popup.clone();
        
        match &mut self.screen {
            AppScreen::MainMenu => {
                self.render_main_menu(ctx, frame);
            }
            AppScreen::TrainWithAi => {
                self.render_train_with_ai(ctx, frame);
            }
            AppScreen::History(HistoryScreenVariant::PastGameSelectionView) => {
                self.render_past_game_selection_view(ctx, frame);
            }
            AppScreen::History(HistoryScreenVariant::GameAnalyzerView(data)) => {
                // Clone the data to avoid multiple mutable borrows
                let data_clone = data.clone();
                self.render_analyzer_view(ctx, frame, &data_clone);
            }
            AppScreen::Multiplayer => {

            }
            AppScreen::Analyze => {

            }
        } // end matchd
        if let Some(popup) = popup_clone {
            self.popup_handler(&popup, ctx, frame);
        }
        }
            

        
    }  

