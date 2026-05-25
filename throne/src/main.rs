use eframe::egui;
use egui::plot::{Line, Plot, PlotPoints};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::fs;
use std::time::{Duration, Instant};

// --- Состояние игры ---
#[derive(PartialEq, Serialize, Deserialize)]
enum GameEvent {
    None,
    Famine,
    Festival,
    BanditAttack,
    Plague,
    Diplomat,
    Earthquake,
}

#[derive(Serialize, Deserialize)]
struct ThroneApp {
    // Ресурсы
    gold: i32,
    guards: u32,
    walls: u32,
    reputation: i32,
    houses: u32,
    
    // Флаги и таймеры
    #[serde(skip)]
    last_event_tick: Instant,
    #[serde(skip)]
    current_event: GameEvent,
    #[serde(skip)]
    event_message: String,
    #[serde(skip)]
    log_messages: Vec<String>,
    #[serde(skip)]
    event_timer: f32,
    
    // Визуальные настройки
    #[serde(skip)]
    show_help: bool,
    #[serde(skip)]
    show_shop: bool,
    #[serde(skip)]
    income_history: Vec<f64>,
}

impl Default for ThroneApp {
    fn default() -> Self {
        let mut app = Self {
            gold: 500,
            guards: 10,
            walls: 1,
            reputation: 50,
            houses: 0,
            last_event_tick: Instant::now(),
            current_event: GameEvent::None,
            event_message: String::new(),
            log_messages: Vec::new(),
            event_timer: 0.0,
            show_help: false,
            show_shop: false,
            income_history: vec![50.0, 55.0, 60.0, 65.0, 70.0],
        };
        app.log_messages.push("✨ Добро пожаловать во дворец, Ваше Величество!".to_string());
        app.load_game();
        app
    }
}

impl ThroneApp {
    fn save_game(&mut self) {
        let save_data = SaveData {
            gold: self.gold,
            guards: self.guards,
            walls: self.walls,
            reputation: self.reputation,
            houses: self.houses,
        };
        
        match serde_json::to_string_pretty(&save_data) {
            Ok(json) => {
                if fs::write("throne_save.json", json).is_ok() {
                    self.log_messages.push("💾 Игра сохранена!".to_string());
                } else {
                    self.log_messages.push("❌ Ошибка сохранения!".to_string());
                }
            }
            Err(_) => {
                self.log_messages.push("❌ Ошибка сериализации!".to_string());
            }
        }
    }
    
    fn load_game(&mut self) {
        if let Ok(data) = fs::read_to_string("throne_save.json") {
            if let Ok(save_data) = serde_json::from_str::<SaveData>(&data) {
                self.gold = save_data.gold;
                self.guards = save_data.guards;
                self.walls = save_data.walls;
                self.reputation = save_data.reputation;
                self.houses = save_data.houses;
                self.log_messages.push("📀 Игра загружена!".to_string());
            }
        }
    }
    
    fn buy_house(&mut self) {
        let cost = 1000;
        if self.gold >= cost {
            self.gold -= cost;
            self.houses += 1;
            self.reputation += 20;
            self.reputation = self.reputation.clamp(0, 100);
            self.log_messages.push(format!("🏠 Построен роскошный дом! Всего домов: {}. Репутация +20!", self.houses));
            self.log_messages.push(format!("💰 Дом приносит +{} золота к налогам!", self.houses * 50));
        } else {
            self.log_messages.push(format!("❌ Недостаточно золота! Нужно 1000 монет, у вас {}", self.gold));
        }
    }
    
    fn collect_taxes(&mut self) {
        let rep_bonus = if self.reputation > 70 { 50 } else if self.reputation < 30 { -50 } else { 0 };
        let wall_bonus = (self.walls * 5) as i32;
        let house_bonus = (self.houses * 10) as i32;
        
        let base_tax = 100;
        let total_bonus_percent = 100 + rep_bonus + wall_bonus + house_bonus;
        let tax_yield = (base_tax * total_bonus_percent) / 100;
        
        self.gold += tax_yield;
        
        self.income_history.push(tax_yield as f64);
        if self.income_history.len() > 20 {
            self.income_history.remove(0);
        }
        
        let msg = format!("💰 Собрано {} золота (Бонус: репутация {}%, стены {}%, дома {}%)", 
                         tax_yield, rep_bonus, wall_bonus, house_bonus);
        self.log_messages.push(msg);
        
        if rand::thread_rng().gen_bool(0.2) {
            self.reputation += 1;
            self.reputation = self.reputation.clamp(0, 100);
            self.log_messages.push("👑 Народ доволен сбором налогов! Репутация +1".to_string());
        }
    }
    
    fn hire_guard(&mut self) {
        let cost = 50;
        if self.gold >= cost {
            self.gold -= cost;
            self.guards += 1;
            self.log_messages.push(format!("⚔️ Нанят стражник! Всего стражников: {}", self.guards));
        } else {
            self.log_messages.push("❌ Недостаточно золота для найма стражи!".to_string());
        }
    }
    
    fn build_wall(&mut self) {
        let cost = 150;
        if self.gold >= cost {
            self.gold -= cost;
            self.walls += 1;
            self.log_messages.push(format!("🧱 Построена стена! Уровень защиты: {}", self.walls));
            self.reputation += 5;
            self.reputation = self.reputation.clamp(0, 100);
            self.log_messages.push("🏆 Стены вдохновляют народ! Репутация +5".to_string());
        } else {
            self.log_messages.push("❌ Недостаточно золота для строительства!".to_string());
        }
    }
    
    fn trigger_random_event(&mut self) {
        let mut rng = rand::thread_rng();
        let event_id = rng.gen_range(0..7);
        
        match event_id {
            0 => {
                let loss = (self.gold / 4).max(50);
                self.gold -= loss;
                self.reputation -= 10;
                self.reputation = self.reputation.clamp(0, 100);
                self.current_event = GameEvent::Famine;
                self.event_message = format!("🌾 Голод охватил деревни! Потеряно {} золота.", loss);
                self.event_timer = 3.0;
            }
            1 => {
                let gain = (self.gold as f32 * 0.3) as i32;
                self.gold += gain;
                self.reputation += 15;
                self.reputation = self.reputation.clamp(0, 100);
                self.current_event = GameEvent::Festival;
                self.event_message = format!("🍎 Праздник урожая! Получено {} золота.", gain);
                self.event_timer = 3.0;
            }
            2 => {
                let attack_power = rng.gen_range(20..50);
                if self.guards as i32 > attack_power {
                    self.log_messages.push("🛡️ Стража отразила атаку бандитов!".to_string());
                    self.reputation += 5;
                    self.reputation = self.reputation.clamp(0, 100);
                    self.current_event = GameEvent::None;
                    self.event_message = "⚔️ Бандиты атаковали, но стража победила!".to_string();
                    self.event_timer = 2.0;
                    return;
                } else {
                    let stolen = rng.gen_range(80..200);
                    self.gold -= stolen;
                    self.reputation -= 20;
                    self.reputation = self.reputation.clamp(0, 100);
                    self.current_event = GameEvent::BanditAttack;
                    self.event_message = format!("🏴‍☠️ Бандиты грабят окраины! Потеряно {} золота.", stolen);
                    self.event_timer = 3.0;
                    
                    let lost_guards = (attack_power / 10).max(1);
                    self.guards = self.guards.saturating_sub(lost_guards);
                    self.log_messages.push(format!("💔 Потеряно {} стражников!", lost_guards));
                }
            }
            3 => {
                let sick_people = rng.gen_range(5..20);
                let cost = sick_people * 10;
                self.gold -= cost;
                self.reputation -= 15;
                self.reputation = self.reputation.clamp(0, 100);
                self.current_event = GameEvent::Plague;
                self.event_message = format!("🦠 Чума в королевстве! Лечение стоит {} золота.", cost);
                self.event_timer = 3.0;
            }
            4 => {
                let gift = rng.gen_range(100..300);
                self.gold += gift;
                self.reputation += 10;
                self.reputation = self.reputation.clamp(0, 100);
                self.current_event = GameEvent::Diplomat;
                self.event_message = format!("🤝 Посол соседнего королевства дарит {} золота!", gift);
                self.event_timer = 3.0;
            }
            5 => {
                let destroyed_walls = (self.walls / 3).max(1);
                self.walls -= destroyed_walls;
                self.reputation -= 10;
                self.reputation = self.reputation.clamp(0, 100);
                self.current_event = GameEvent::Earthquake;
                self.event_message = format!("🌋 Землетрясение разрушило {} стен!", destroyed_walls);
                self.event_timer = 3.0;
            }
            _ => {}
        }
        self.log_messages.push(self.event_message.clone());
    }
    
    fn render_income_chart(&mut self, ui: &mut egui::Ui) {
        ui.label("📈 График доходов (последние 20 сборов):");
        let points: PlotPoints = self.income_history
            .iter()
            .enumerate()
            .map(|(i, &value)| [i as f64, value])
            .collect();
        
        let line = Line::new(points)
            .color(egui::Color32::GOLD)
            .width(2.0);
        
        Plot::new("income_chart")
            .height(150.0)
            .width_filled()
            .show(ui, |plot_ui| {
                plot_ui.line(line);
            });
    }
    
    fn render_ui(&mut self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 15, 25);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 35, 55);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 55, 80);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(255, 215, 0);
        ctx.set_style(style);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // Верхняя панель ресурсов
            ui.horizontal(|ui| {
                ui.heading("🏰 THRONE");
                ui.add_space(20.0);
                ui.heading(format!("💰 {} золота", self.gold));
                ui.separator();
                ui.heading(format!("⚔️ {} стражников", self.guards));
                ui.separator();
                ui.heading(format!("🧱 {} стен", self.walls));
                ui.separator();
                ui.heading(format!("🏠 {} домов", self.houses));
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("👑 Репутация:");
                    let rep_color = if self.reputation > 70 { egui::Color32::GREEN } 
                                    else if self.reputation < 30 { egui::Color32::RED }
                                    else { egui::Color32::YELLOW };
                    ui.colored_label(rep_color, format!("{}", self.reputation));
                    
                    let progress = self.reputation as f32 / 100.0;
                    let progress_bar = egui::ProgressBar::new(progress)
                        .desired_width(100.0)
                        .show_percentage();
                    ui.add(progress_bar);
                });
            });
            
            ui.add_space(20.0);
            ui.separator();
            
            // Секция действий
            ui.horizontal(|ui| {
                ui.heading("📜 ДЕЙСТВИЯ");
                ui.add_space(30.0);
                
                if ui.button("💰 Собрать налоги").clicked() {
                    self.collect_taxes();
                }
                if ui.button("⚔️ Нанять стража (50💰)").clicked() {
                    self.hire_guard();
                }
                if ui.button("🧱 Построить стену (150💰)").clicked() {
                    self.build_wall();
                }
                if ui.button("🏠 Магазин домов (1000💰)").clicked() {
                    self.show_shop = !self.show_shop;
                }
                if ui.button("💾 Сохранить").clicked() {
                    self.save_game();
                }
                if ui.button("📀 Загрузить").clicked() {
                    self.load_game();
                }
                if ui.button("❓ Помощь").clicked() {
                    self.show_help = !self.show_help;
                }
            });
            
            ui.add_space(10.0);
            
            // Отображение активного события
            if self.current_event != GameEvent::None {
                let event_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(100, 50, 50, 100))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
                    .rounding(5.0);
                event_frame.show(ui, |ui| {
                    ui.label(format!("⚠️ СОБЫТИЕ: {}", self.event_message));
                });
                
                self.event_timer -= ui.input(|i| i.unstable_dt);
                if self.event_timer <= 0.0 {
                    self.current_event = GameEvent::None;
                }
            }
            
            ui.separator();
            
            // График доходов
            self.render_income_chart(ui);
            ui.separator();
            
            // Лог событий
            ui.heading("📜 ЛЕТОПИСЬ");
            let mut text = String::new();
            let start = if self.log_messages.len() > 10 { self.log_messages.len() - 10 } else { 0 };
            for msg in &self.log_messages[start..] {
                text.push_str(msg);
                text.push('\n');
            }
            
            let text_edit = egui::TextEdit::multiline(&mut text.as_str())
                .font(egui::TextStyle::Monospace)
                .desired_rows(8)
                .desired_width(f32::INFINITY)
                .interactive(false);
            ui.add(text_edit);
            
            // Окно магазина
            if self.show_shop {
                egui::Window::new("🏠 Магазин готовых домов")
                    .collapsible(false)
                    .resizable(false)
                    .default_size([350.0, 250.0])
                    .show(ctx, |ui| {
                        ui.label("✨ Эксклюзивные готовые дома для знати ✨");
                        ui.add_space(10.0);
                        ui.label("🏰 Королевский особняк");
                        ui.label("💰 Цена: 1000 золотых монет");
                        ui.label("📈 Преимущества:");
                        ui.label("  • +20 к репутации");
                        ui.label("  • +10% к налогам за каждый дом");
                        ui.label("  • Престиж и уважение народа");
                        ui.add_space(20.0);
                        
                        ui.horizontal(|ui| {
                            if ui.button("🏠 Купить дом (1000💰)").clicked() {
                                self.buy_house();
                                self.show_shop = false;
                            }
                            if ui.button("Закрыть").clicked() {
                                self.show_shop = false;
                            }
                        });
                        
                        ui.add_space(10.0);
                        ui.label(format!("💰 Ваше золото: {}", self.gold));
                        ui.label(format!("🏠 Уже построено домов: {}", self.houses));
                    });
            }
            
            // Окно помощи
            if self.show_help {
                egui::Window::new("❓ Советы по управлению")
                    .collapsible(false)
                    .resizable(false)
                    .default_size([350.0, 300.0])
                    .show(ctx, |ui| {
                        ui.label("📖 Как играть:");
                        ui.label("- Собирайте налоги для получения золота.");
                        ui.label("- Нанимайте стражников для защиты.");
                        ui.label("- Стройте стены: они повышают доход и репутацию.");
                        ui.label("- Покупайте готовые дома за 1000 монет.");
                        ui.label("  Дома дают большой бонус к налогам!");
                        ui.label("- Следите за репутацией: она влияет на налоги.");
                        ui.label("- Каждые 15 секунд случаются случайные события.");
                        ui.label("- Используйте кнопки Сохранить/Загрузить.");
                        ui.add_space(10.0);
                        ui.label("💡 Совет: сначала купите несколько домов,");
                        ui.label("  они окупятся за счет повышенных налогов!");
                        ui.add_space(10.0);
                        if ui.button("Закрыть").clicked() {
                            self.show_help = false;
                        }
                    });
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    gold: i32,
    guards: u32,
    walls: u32,
    reputation: i32,
    houses: u32,
}

impl eframe::App for ThroneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_event_tick.elapsed() > Duration::from_secs(15) {
            self.trigger_random_event();
            self.last_event_tick = Instant::now();
        }
        
        self.render_ui(ctx);
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

// --- Точка входа (ИСПРАВЛЕНО) ---
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 750.0])
            .with_min_inner_size([700.0, 600.0])
            .with_title("Throne — Управление королевством Deluxe"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Throne Game",
        options,
        Box::new(|_cc| Box::new(ThroneApp::default())),
    )
}
