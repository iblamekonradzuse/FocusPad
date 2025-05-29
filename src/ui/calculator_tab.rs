use crate::app::StatusMessage;
use eframe::egui::{self, Button, Color32, RichText, Ui, Vec2};
use std::str::FromStr;

thread_local! {
    static DISPLAY: std::cell::RefCell<String> = std::cell::RefCell::new(String::from("0"));
    static OPERAND1: std::cell::RefCell<Option<f64>> = std::cell::RefCell::new(None);
    static OPERATION: std::cell::RefCell<Option<Operation>> = std::cell::RefCell::new(None);
    static NEW_INPUT: std::cell::RefCell<bool> = std::cell::RefCell::new(true);
    static MEMORY: std::cell::RefCell<f64> = std::cell::RefCell::new(0.0);
    static ANGLE_MODE: std::cell::RefCell<AngleMode> = std::cell::RefCell::new(AngleMode::Degrees);
    static LOG_BASE: std::cell::RefCell<f64> = std::cell::RefCell::new(10.0); // Default log base
}

#[derive(Clone, Copy, PartialEq)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Root,
    CustomLog,
}

#[derive(Clone, Copy, PartialEq)]
enum AngleMode {
    Degrees,
    Radians,
}

impl AngleMode {
    fn convert_to_radians(&self, value: f64) -> f64 {
        match self {
            AngleMode::Degrees => value.to_radians(),
            AngleMode::Radians => value,
        }
    }
}

pub fn display(ui: &mut Ui, status: &mut StatusMessage) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.heading("Calculator");

        ui.add_space(10.0);

        display_calculator(ui, status);
    });
}

fn display_calculator(ui: &mut Ui, status: &mut StatusMessage) {
    // Display value with improved styling
    DISPLAY.with(|display| {
        let display_text = display.borrow();
        ui.add_space(5.0);
        ui.add(
            egui::TextEdit::singleline(&mut display_text.as_str())
                .font(egui::TextStyle::Monospace)
                .text_color(Color32::WHITE)
                .frame(true)
                .desired_width(ui.available_width() - 20.0)
                .min_size(egui::vec2(ui.available_width() - 20.0, 40.0)),
        );
    });

    ui.add_space(10.0);

    // Angle mode selector with better styling
    ANGLE_MODE.with(|mode| {
        let mut current_mode = *mode.borrow();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Angle:").text_style(egui::TextStyle::Button));
            if ui
                .selectable_label(current_mode == AngleMode::Degrees, "Degrees")
                .clicked()
            {
                current_mode = AngleMode::Degrees;
                *mode.borrow_mut() = current_mode;
            }
            if ui
                .selectable_label(current_mode == AngleMode::Radians, "Radians")
                .clicked()
            {
                current_mode = AngleMode::Radians;
                *mode.borrow_mut() = current_mode;
            }
        });
    });

    // Custom log base input
    ui.horizontal(|ui| {
        ui.label(RichText::new("Log Base:").text_style(egui::TextStyle::Button));
        LOG_BASE.with(|base| {
            let mut current_base = *base.borrow();
            let response = ui.add(
                egui::DragValue::new(&mut current_base)
                    .speed(0.1)
                    .clamp_range(2.0..=100.0),
            );
            if response.changed() {
                *base.borrow_mut() = current_base;
                status.show(&format!("Log base set to {}", current_base));
            }
        });

        if ui.button("logₙ(x)").clicked() {
            LOG_BASE.with(|base| {
                let current_base = *base.borrow();
                apply_scientific_function(|x, _| x.log(current_base), status);
            });
        }
    });

    ui.add_space(10.0);

    // Calculator grid with uniform button sizes and better styling
    let button_size = Vec2::new(60.0, 45.0);

    // Memory functions
    ui.horizontal(|ui| {
        calc_button(ui, "MC", button_size, || {
            MEMORY.with(|memory| {
                *memory.borrow_mut() = 0.0;
            });
            status.show("Memory cleared");
        });

        calc_button(ui, "MR", button_size, || {
            MEMORY.with(|memory| {
                let value = *memory.borrow();
                DISPLAY.with(|display| {
                    *display.borrow_mut() = format!("{}", value);
                });
                NEW_INPUT.with(|new_input| {
                    *new_input.borrow_mut() = true;
                });
            });
        });

        calc_button(ui, "M+", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    MEMORY.with(|memory| {
                        *memory.borrow_mut() += value;
                        status.show(&format!("Added to memory: {}", *memory.borrow()));
                    });
                }
            });
        });

        calc_button(ui, "M-", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    MEMORY.with(|memory| {
                        *memory.borrow_mut() -= value;
                        status.show(&format!("Subtracted from memory: {}", *memory.borrow()));
                    });
                }
            });
        });
    });

    ui.add_space(5.0);

    // Scientific functions
    ui.horizontal(|ui| {
        calc_button(ui, "sin", button_size, || {
            apply_scientific_function(|x, mode| mode.convert_to_radians(x).sin(), status);
        });

        calc_button(ui, "cos", button_size, || {
            apply_scientific_function(|x, mode| mode.convert_to_radians(x).cos(), status);
        });

        calc_button(ui, "tan", button_size, || {
            apply_scientific_function(|x, mode| mode.convert_to_radians(x).tan(), status);
        });

        calc_button(ui, "ln", button_size, || {
            apply_scientific_function(|x, _| x.ln(), status);
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "asin", button_size, || {
            apply_scientific_function(
                |x, mode| match mode {
                    AngleMode::Degrees => x.asin() * 180.0 / std::f64::consts::PI,
                    AngleMode::Radians => x.asin(),
                },
                status,
            );
        });

        calc_button(ui, "acos", button_size, || {
            apply_scientific_function(
                |x, mode| match mode {
                    AngleMode::Degrees => x.acos() * 180.0 / std::f64::consts::PI,
                    AngleMode::Radians => x.acos(),
                },
                status,
            );
        });

        calc_button(ui, "atan", button_size, || {
            apply_scientific_function(
                |x, mode| match mode {
                    AngleMode::Degrees => x.atan() * 180.0 / std::f64::consts::PI,
                    AngleMode::Radians => x.atan(),
                },
                status,
            );
        });

        calc_button(ui, "log₁₀", button_size, || {
            apply_scientific_function(|x, _| x.log10(), status);
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "x²", button_size, || {
            apply_scientific_function(|x, _| x * x, status);
        });

        calc_button(ui, "√x", button_size, || {
            apply_scientific_function(|x, _| x.sqrt(), status);
        });

        calc_button(ui, "xʸ", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    OPERAND1.with(|op1| {
                        *op1.borrow_mut() = Some(value);
                    });
                    OPERATION.with(|op| {
                        *op.borrow_mut() = Some(Operation::Power);
                    });
                    NEW_INPUT.with(|new_input| {
                        *new_input.borrow_mut() = true;
                    });
                }
            });
        });

        calc_button(ui, "ʸ√x", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    OPERAND1.with(|op1| {
                        *op1.borrow_mut() = Some(value);
                    });
                    OPERATION.with(|op| {
                        *op.borrow_mut() = Some(Operation::Root);
                    });
                    NEW_INPUT.with(|new_input| {
                        *new_input.borrow_mut() = true;
                    });
                }
            });
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "1/x", button_size, || {
            apply_scientific_function(|x, _| 1.0 / x, status);
        });

        calc_button(ui, "eˣ", button_size, || {
            apply_scientific_function(|x, _| x.exp(), status);
        });

        calc_button(ui, "π", button_size, || {
            DISPLAY.with(|display| {
                *display.borrow_mut() = std::f64::consts::PI.to_string();
                NEW_INPUT.with(|new_input| {
                    *new_input.borrow_mut() = true;
                });
            });
        });

        calc_button(ui, "e", button_size, || {
            DISPLAY.with(|display| {
                *display.borrow_mut() = std::f64::consts::E.to_string();
                NEW_INPUT.with(|new_input| {
                    *new_input.borrow_mut() = true;
                });
            });
        });
    });

    ui.add_space(10.0);

    // Custom log base operation
    ui.horizontal(|ui| {
        calc_button(ui, "logₙ(base)", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    LOG_BASE.with(|base| {
                        *base.borrow_mut() = value;
                        status.show(&format!("Log base set to {}", value));
                    });
                    NEW_INPUT.with(|new_input| {
                        *new_input.borrow_mut() = true;
                    });
                }
            });
        });

        calc_button(ui, "base^x", button_size, || {
            LOG_BASE.with(|base| {
                let current_base = *base.borrow();
                apply_scientific_function(|x, _| current_base.powf(x), status);
            });
        });

        calc_button(ui, "%", button_size, || {
            apply_scientific_function(|x, _| x / 100.0, status);
        });

        calc_button(ui, "mod", button_size, || {
            DISPLAY.with(|display| {
                if let Ok(value) = f64::from_str(&display.borrow()) {
                    OPERAND1.with(|op1| {
                        *op1.borrow_mut() = Some(value);
                    });
                    OPERATION.with(|op| {
                        *op.borrow_mut() = Some(Operation::CustomLog);
                    });
                    NEW_INPUT.with(|new_input| {
                        *new_input.borrow_mut() = true;
                    });
                }
            });
        });
    });

    ui.add_space(10.0);

    // Basic Calculator grid
    ui.horizontal(|ui| {
        calc_button(ui, "7", button_size, || {
            add_digit('7');
        });
        calc_button(ui, "8", button_size, || {
            add_digit('8');
        });
        calc_button(ui, "9", button_size, || {
            add_digit('9');
        });
        calc_button(ui, "÷", button_size, || {
            set_operation(Operation::Divide);
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "4", button_size, || {
            add_digit('4');
        });
        calc_button(ui, "5", button_size, || {
            add_digit('5');
        });
        calc_button(ui, "6", button_size, || {
            add_digit('6');
        });
        calc_button(ui, "×", button_size, || {
            set_operation(Operation::Multiply);
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "1", button_size, || {
            add_digit('1');
        });
        calc_button(ui, "2", button_size, || {
            add_digit('2');
        });
        calc_button(ui, "3", button_size, || {
            add_digit('3');
        });
        calc_button(ui, "-", button_size, || {
            set_operation(Operation::Subtract);
        });
    });

    ui.horizontal(|ui| {
        calc_button(ui, "0", button_size, || {
            add_digit('0');
        });
        calc_button(ui, ".", button_size, || {
            DISPLAY.with(|display| {
                let mut display_text = display.borrow_mut();
                NEW_INPUT.with(|new_input| {
                    if *new_input.borrow() {
                        *display_text = "0.".to_string();
                        *new_input.borrow_mut() = false;
                    } else if !display_text.contains('.') {
                        display_text.push('.');
                    }
                });
            });
        });
        calc_button(ui, "=", button_size, || {
            calculate_result(status);
        });
        calc_button(ui, "+", button_size, || {
            set_operation(Operation::Add);
        });
    });

    ui.add_space(5.0);

    // Additional controls
    ui.horizontal(|ui| {
        calc_button(ui, "C", button_size, || {
            DISPLAY.with(|display| {
                *display.borrow_mut() = "0".to_string();
            });
            OPERAND1.with(|op1| {
                *op1.borrow_mut() = None;
            });
            OPERATION.with(|op| {
                *op.borrow_mut() = None;
            });
            NEW_INPUT.with(|new_input| {
                *new_input.borrow_mut() = true;
            });
        });

        calc_button(ui, "CE", button_size, || {
            DISPLAY.with(|display| {
                *display.borrow_mut() = "0".to_string();
            });
            NEW_INPUT.with(|new_input| {
                *new_input.borrow_mut() = true;
            });
        });

        calc_button(ui, "⌫", button_size, || {
            DISPLAY.with(|display| {
                let mut display_text = display.borrow_mut();
                if display_text.len() > 1 {
                    display_text.pop();
                } else {
                    *display_text = "0".to_string();
                    NEW_INPUT.with(|new_input| {
                        *new_input.borrow_mut() = true;
                    });
                }
            });
        });

        calc_button(ui, "±", button_size, || {
            DISPLAY.with(|display| {
                let mut display_text = display.borrow_mut();
                if let Ok(value) = f64::from_str(&display_text) {
                    *display_text = format!("{}", -value);
                }
            });
        });
    });
}

// Helper function for styled calculator buttons
fn calc_button<F>(ui: &mut Ui, text: &str, size: Vec2, on_click: F)
where
    F: FnOnce(),
{
    let button = ui.add_sized(
        size,
        Button::new(RichText::new(text).size(18.0)).fill(Color32::from_rgb(60, 60, 60)),
    );

    if button.clicked() {
        on_click();
    }

    // Add hover effect
    if button.hovered() {
        button.highlight();
    }
}

fn add_digit(digit: char) {
    DISPLAY.with(|display| {
        let mut display_text = display.borrow_mut();
        NEW_INPUT.with(|new_input| {
            if *new_input.borrow() {
                *display_text = digit.to_string();
                *new_input.borrow_mut() = false;
            } else if *display_text == "0" {
                *display_text = digit.to_string();
            } else {
                display_text.push(digit);
            }
        });
    });
}

fn set_operation(op: Operation) {
    DISPLAY.with(|display| {
        if let Ok(value) = f64::from_str(&display.borrow()) {
            OPERAND1.with(|op1| {
                *op1.borrow_mut() = Some(value);
            });
            OPERATION.with(|operation| {
                *operation.borrow_mut() = Some(op);
            });
            NEW_INPUT.with(|new_input| {
                *new_input.borrow_mut() = true;
            });
        }
    });
}

fn calculate_result(status: &mut StatusMessage) {
    let mut result = 0.0;
    let mut error = false;

    OPERAND1.with(|op1| {
        if let Some(operand1) = *op1.borrow() {
            DISPLAY.with(|display| {
                if let Ok(operand2) = f64::from_str(&display.borrow()) {
                    OPERATION.with(|op| {
                        if let Some(operation) = *op.borrow() {
                            result = match operation {
                                Operation::Add => operand1 + operand2,
                                Operation::Subtract => operand1 - operand2,
                                Operation::Multiply => operand1 * operand2,
                                Operation::Divide => {
                                    if operand2 == 0.0 {
                                        error = true;
                                        0.0
                                    } else {
                                        operand1 / operand2
                                    }
                                }
                                Operation::Power => operand1.powf(operand2),
                                Operation::Root => {
                                    if operand2 == 0.0 {
                                        error = true;
                                        0.0
                                    } else {
                                        operand1.powf(1.0 / operand2)
                                    }
                                }
                                Operation::CustomLog => {
                                    if operand1 <= 0.0 || operand2 <= 0.0 || operand2 == 1.0 {
                                        error = true;
                                        0.0
                                    } else {
                                        // log_a(b) = ln(b) / ln(a)
                                        operand1.log(operand2)
                                    }
                                }
                            };
                        }
                    });
                }
            });
        }
    });

    if error {
        status.show("Error: Invalid operation");
        DISPLAY.with(|display| {
            *display.borrow_mut() = "Error".to_string();
        });
    } else {
        DISPLAY.with(|display| {
            *display.borrow_mut() = format!("{}", result);
        });

        // Reset operation state
        OPERAND1.with(|op1| {
            *op1.borrow_mut() = None;
        });
        OPERATION.with(|op| {
            *op.borrow_mut() = None;
        });
    }

    NEW_INPUT.with(|new_input| {
        *new_input.borrow_mut() = true;
    });
}

fn apply_scientific_function<F>(func: F, status: &mut StatusMessage)
where
    F: Fn(f64, AngleMode) -> f64,
{
    let mut error = false;
    let mut result = 0.0;

    DISPLAY.with(|display| {
        if let Ok(value) = f64::from_str(&display.borrow()) {
            ANGLE_MODE.with(|mode| {
                let angle_mode = *mode.borrow();
                result = func(value, angle_mode);
                if result.is_nan() || result.is_infinite() {
                    error = true;
                }
            });
        } else {
            error = true;
        }
    });

    if error {
        status.show("Error: Invalid operation");
        DISPLAY.with(|display| {
            *display.borrow_mut() = "Error".to_string();
        });
    } else {
        DISPLAY.with(|display| {
            *display.borrow_mut() = format!("{}", result);
        });
    }

    NEW_INPUT.with(|new_input| {
        *new_input.borrow_mut() = true;
    });
}
