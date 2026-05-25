use eframe::egui;
use rand::Rng;
use std::time::{Duration, Instant};

// --- Состояние игры ---
#[derive(PartialEq)]
enum GameEvent {
    None,
    Famine,       // Голод
    Festival,     // Праздник урожая
    BanditAttack, // Атака бандитов
}

struct ThroneApp {
    // Ресурсы
    gold: i32,
    guards: u32,
    walls: u32,
    reputation: i32, // Влияет на налоги и события
    
    // Флаги и таймеры
    last_event_tick: Instant,
    current_event: GameEvent,
    event_message: String,
    log_messages: Vec<String>, // История действий
    
    // Визуальные настройки
    show_help: bool,
}

impl Default for ThroneApp {
    fn default() -> Self {
        let mut app = Self {
            gold: 500,
            guards: 10,
            walls: 1,
            reputation: 50,
            last_event_tick: Instant::now(),
            current_event: GameEvent::None,
            event_message: String::new(),
            log_messages: Vec::new(),
            show_help: false,
        };
        app.log_messages.push("✨ Добро пожаловать во дворец, Ваше Величество!".to_string());
        app
    }
}

impl ThroneApp {
    /// Основная игровая логика: налоги с бонусом от стен и репутации
    fn collect_taxes(&mut self) {
        // Репутация выше 70 дает +50% налогов, ниже 30 дает -50%
        let rep_bonus = if self.reputation > 70 { 50 } else if self.reputation < 30 { -50 } else { 0 };
        let wall_bonus = (self.walls * 5) as i32; // Каждая стена дает +5% к сбору
        
        let base_tax = 100;
        let total_bonus_percent = 100 + rep_bonus + wall_bonus;
        let tax_yield = (base_tax * total_bonus_percent) / 100;
        
        self.gold += tax_yield;
        let msg = format!("💰 Собрано {} золота (Бонус: репутация {}%, стены {}%)", 
                         tax_yield, rep_bonus, wall_bonus);
        self.log_messages.push(msg);
        
        // Небольшой шанс роста репутации за заботу о налогах
        if rand::thread_rng().gen_bool(0.2) {
            self.reputation += 1;
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
            // Стены повышают безопасность и репутацию
            self.reputation += 5;
            self.log_messages.push("🏆 Стены вдохновляют народ! Репутация +5".to_string());
        } else {
            self.log_messages.push("❌ Недостаточно золота для строительства!".to_string());
        }
    }
    
    /// Обработка случайных событий (вызывается каждые ~15 секунд)
    fn trigger_random_event(&mut self) {
        let mut rng = rand::thread_rng();
        let event_id = rng.gen_range(0..3);
        
        match event_id {
            0 => { // Голод
                let loss = (self.gold / 4).max(50);
                self.gold -= loss;
                self.reputation -= 10;
                self.current_event = GameEvent::Famine;
                self.event_message = format!("🌾 Голод охватил деревни! Потеряно {} золота.", loss);
            }
            1 => { // Праздник урожая
                let gain = (self.gold as f32 * 0.3) as i32;
                self.gold += gain;
                self.reputation += 15;
                self.current_event = GameEvent::Festival;
                self.event_message = format!("🍎 Праздник урожая! Получено {} золота.", gain);
            }
            2 => { // Атака бандитов (зависит от стражников)
                let attack_power = rng.gen_range(20..50);
                if self.guards as i32 > attack_power {
                    self.log_messages.push("🛡️ Стража отразила атаку бандитов!".to_string());
                    self.reputation += 5;
                    self.current_event = GameEvent::None;
                    self.event_message = "⚔️ Бандиты атаковали, но стража победила!".to_string();
                    return;
                } else {
                    let stolen = rng.gen_range(80..200);
                    self.gold -= stolen;
                    self.reputation -= 20;
                    self.current_event = GameEvent::BanditAttack;
                    self.event_message = format!("🏴‍☠️ Бандиты грабят окраины! Потеряно {} золота.", stolen);
                }
            }
            _ => {}
        }
        self.log_messages.push(self.event_message.clone());
    }

    /// Рендер главного окна (GUI)
    fn render_ui(&mut self, ctx: &egui::Context) {
        // Настройка стиля: темная тема с золотыми акцентами
        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 15, 25);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 35, 55);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 55, 80);
        style.visuals.hyperlink_color = egui::Color32::from_rgb(255, 215, 0);
        ctx.set_style(style);
        
        // Центральная панель с отступами
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // --- Верхняя панель ресурсов ---
            ui.horizontal(|ui| {
                ui.label("🏰").heading();
                ui.heading("THRONE");
                ui.add_space(20.0);
                ui.label(format!("💰 {} золота", self.gold)).heading();
                ui.separator();
                ui.label(format!("⚔️ {} стражников", self.guards)).heading();
                ui.separator();
                ui.label(format!("🧱 {} стен", self.walls)).heading();
                ui.separator();
                
                // Полоса репутации
                ui.horizontal(|ui| {
                    ui.label("👑 Репутация:");
                    let rep_color = if self.reputation > 70 { egui::Color32::GREEN } 
                                    else if self.reputation < 30 { egui::Color32::RED }
                                    else { egui::Color32::YELLOW };
                    ui.colored_label(rep_color, format!("{}", self.reputation));
                    
                    // Индикатор прогресса
                    let progress = self.reputation as f32 / 100.0;
                    let progress_bar = egui::ProgressBar::new(progress)
                        .desired_width(100.0)
                        .show_percentage();
                    ui.add(progress_bar);
                });
            });
            
            ui.add_space(20.0);
            ui.separator();
            
            // --- Секция действий (Кнопки) ---
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
                if ui.button("❓ Помощь").clicked() {
                    self.show_help = !self.show_help;
                }
            });
            
            ui.add_space(10.0);
            
            // --- Отображение активного события (если есть) ---
            if self.current_event != GameEvent::None {
                let event_frame = egui::Frame::none()
                    .fill(egui::Color32::from_rgba_unmultiplied(100, 50, 50, 100))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
                    .rounding(5.0);
                event_frame.show(ui, |ui| {
                    ui.label(format!("⚠️ СОБЫТИЕ: {}", self.event_message));
                });
                ui.add_space(10.0);
                // Сбрасываем событие, чтобы сообщение не висело вечно
                if ui.button("Понятно").clicked() {
                    self.current_event = GameEvent::None;
                }
            }
            
            ui.separator();
            
            // --- Лог событий (история) ---
            ui.heading("📜 ЛЕТОПИСЬ");
            let mut text = String::new();
            // Показываем последние 10 сообщений
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
            
            // --- Окно помощи (модальное) ---
            if self.show_help {
                egui::Window::new("❓ Советы по управлению")
                    .collapsible(false)
                    .resizable(false)
                    .default_size([300.0, 200.0])
                    .show(ctx, |ui| {
                        ui.label("📖 Как играть:");
                        ui.label("- Собирайте налоги для получения золота.");
                        ui.label("- Нанимайте стражников для защиты.");
                        ui.label("- Стройте стены: они повышают доход и репутацию.");
                        ui.label("- Следите за репутацией: она влияет на налоги.");
                        ui.label("- Каждые 15 секунд случаются случайные события.");
                        ui.add_space(10.0);
                        if ui.button("Закрыть").clicked() {
                            self.show_help = false;
                        }
                    });
            }
        });
    }
}

impl eframe::App for ThroneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Автоматический таймер событий (каждые 15 секунд)
        if self.last_event_tick.elapsed() > Duration::from_secs(15) {
            self.trigger_random_event();
            self.last_event_tick = Instant::now();
        }
        
        self.render_ui(ctx);
        
        // Запрашиваем постоянный перерисовку для анимации таймера (опционально)
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

// --- Точка входа ---
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 450.0])
            .with_title("Throne — Управление королевством"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Throne Game",
        options,
        Box::new(|_cc| Box::new(ThroneApp::default())),
    )
}
