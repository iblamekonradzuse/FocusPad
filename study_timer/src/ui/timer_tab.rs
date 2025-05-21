use crate::app::StatusMessage;
use crate::data::StudyData;
use crate::debug::DebugTools;
use crate::timer::Timer;
use chrono::Local;
use eframe::egui::{self, Ui};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

thread_local! {
    static DESCRIPTION: std::cell::RefCell<String> = std::cell::RefCell::new(String::new());
    static CUSTOM_BREAK_MINUTES: std::cell::RefCell<String> = std::cell::RefCell::new(String::from("15"));
    static BREAK_END_TIME: std::cell::RefCell<Option<Instant>> = std::cell::RefCell::new(None);
    static ALARM_VOLUME: std::cell::RefCell<f32> = std::cell::RefCell::new(0.8);
    static ALARM_PATH: std::cell::RefCell<String> = std::cell::RefCell::new(String::from("assets/alarm.mp3"));
    // Keep track of any currently playing alarm thread
    static AUDIO_THREAD_HANDLE: std::cell::RefCell<Option<std::thread::JoinHandle<()>>> = std::cell::RefCell::new(None);
    // Store current playing audio process
    static AUDIO_PROCESS: std::cell::RefCell<Option<Child>> = std::cell::RefCell::new(None);
    // Flag to indicate if alarm is currently playing
    static ALARM_PLAYING: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
}

pub fn display(
    ui: &mut Ui,
    timer: &mut Timer,
    study_data: &mut StudyData,
    debug_tools: &mut DebugTools,
    status: &mut StatusMessage,
) {
    let elapsed_minutes = timer.get_elapsed_minutes();
    let hours = (elapsed_minutes as i32) / 60;
    let minutes = (elapsed_minutes as i32) % 60;
    let seconds = ((elapsed_minutes * 60.0) as i32) % 60;

    // Check if break timer has ended
    let break_ended = BREAK_END_TIME.with(|break_end_time| {
        let mut break_end_cell = break_end_time.borrow_mut();
        if let Some(end_time) = *break_end_cell {
            if end_time <= Instant::now() {
                // Break has ended
                *break_end_cell = None;
                return true;
            }
        }
        false
    });

    if break_ended {
        // Play alarm sound
        if play_alarm_sound() {
            status.show("🔔 Break ended! Time to study again!");
        } else {
            status.show("🔔 Break ended! (Failed to play alarm)");
        }

        // Auto-start the timer again if it was paused
        if !timer.is_running {
            timer.start();
        }
    }

    // Request frequent repaint if on break to update the timer display
    if BREAK_END_TIME.with(|break_end_time| break_end_time.borrow().is_some()) {
        ui.ctx().request_repaint();
    }

    // Display the timer in large font
    ui.vertical_centered(|ui| {
        ui.add_space(20.0);

        // Show break countdown if on break
        let _on_break = BREAK_END_TIME.with(|break_end_time| {
            if let Some(end_time) = *break_end_time.borrow() {
                let remaining = end_time.duration_since(Instant::now());
                let mins = remaining.as_secs() / 60;
                let secs = remaining.as_secs() % 60;

                ui.heading(format!("BREAK: {:02}:{:02} remaining", mins, secs));
                true
            } else {
                ui.heading(format!("{:02}:{:02}:{:02}", hours, minutes, seconds));
                false
            }
        });

        ui.add_space(30.0);
    });

    // Check if alarm is playing and show stop button if needed
    let alarm_playing = ALARM_PLAYING.with(|playing| *playing.borrow());
    if alarm_playing {
        ui.vertical_centered(|ui| {
            if ui.button("🔕 Stop Alarm").clicked() {
                stop_alarm_sound();
                status.show("Alarm stopped");
            }
        });
        ui.add_space(10.0);
    }

    // Optional description for the session
    DESCRIPTION.with(|description| {
        let mut description = description.borrow_mut();
        ui.horizontal(|ui| {
            ui.label("Description (optional):");
            ui.text_edit_singleline(&mut *description);
        });
    });

    ui.add_space(10.0);

    // Control buttons
    ui.horizontal(|ui| {
        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                if timer.is_running {
                    if ui.button("⏸ Pause").clicked() {
                        timer.pause();
                        status.show("Timer paused");
                    }
                } else {
                    if ui.button("▶ Start").clicked() {
                        timer.start();
                        status.show("Timer started");
                    }
                }

                if ui.button("💾 Save").clicked() {
                    let minutes = timer.get_elapsed_minutes();
                    if minutes > 0.0 {
                        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

                        // Get the description
                        let description = DESCRIPTION.with(|desc| {
                            let desc = desc.borrow();
                            if desc.is_empty() {
                                None
                            } else {
                                Some(desc.clone())
                            }
                        });

                        if let Err(e) = study_data.add_session(today, minutes, description) {
                            status.show(&format!("Error saving: {}", e));
                        } else {
                            status.show(&format!("Saved {:.1} minutes to today's total", minutes));
                            // Reset accumulated time but keep running if it was running
                            let was_running = timer.is_running;
                            timer.reset();
                            if was_running {
                                timer.start();
                            }

                            // Clear description
                            DESCRIPTION.with(|desc| {
                                *desc.borrow_mut() = String::new();
                            });
                        }
                    } else {
                        status.show("No time to save");
                    }
                }

                if ui.button("⏹ Stop").clicked() {
                    let minutes = timer.get_elapsed_minutes();
                    if minutes > 0.0 {
                        let today = Local::now().date_naive().format("%Y-%m-%d").to_string();

                        // Get the description
                        let description = DESCRIPTION.with(|desc| {
                            let desc = desc.borrow();
                            if desc.is_empty() {
                                None
                            } else {
                                Some(desc.clone())
                            }
                        });

                        if let Err(e) = study_data.add_session(today, minutes, description) {
                            status.show(&format!("Error saving: {}", e));
                        } else {
                            status.show(&format!("Saved {:.1} minutes to today's total", minutes));

                            // Clear description
                            DESCRIPTION.with(|desc| {
                                *desc.borrow_mut() = String::new();
                            });
                        }
                    }
                    timer.reset();
                    status.show("Timer stopped and reset");
                }
            },
        );
    });

    // Break section
    ui.add_space(15.0);
    ui.separator();
    ui.heading("Take a Break");

    let on_break = BREAK_END_TIME.with(|break_end_time| break_end_time.borrow().is_some());

    if on_break {
        // Show cancel button if on break
        if ui.button("⏹ Cancel Break").clicked() {
            BREAK_END_TIME.with(|break_end_time| {
                *break_end_time.borrow_mut() = None;
            });
            status.show("Break cancelled");
        }
    } else {
        // Break buttons row
        ui.horizontal(|ui| {
            if ui.button("☕ 5 min").clicked() {
                start_break(5, status);
                if timer.is_running {
                    timer.pause();
                    status.show("Timer paused. Break started for 5 minutes");
                }
            }

            if ui.button("🍵 10 min").clicked() {
                start_break(10, status);
                if timer.is_running {
                    timer.pause();
                    status.show("Timer paused. Break started for 10 minutes");
                }
            }

            CUSTOM_BREAK_MINUTES.with(|mins| {
                let mut mins_str = mins.borrow_mut();
                ui.horizontal(|ui| {
                    // Simple approach - just use text_edit_singleline without size constraints
                    ui.add(egui::TextEdit::singleline(&mut *mins_str).desired_width(60.0));

                    if ui.button("Custom").clicked() {
                        match mins_str.parse::<u64>() {
                            Ok(m) if m > 0 => {
                                start_break(m, status);
                                if timer.is_running {
                                    timer.pause();
                                    status.show(&format!(
                                        "Timer paused. Break started for {} minutes",
                                        m
                                    ));
                                }
                            }
                            _ => {
                                status.show("Please enter a valid number of minutes");
                            }
                        }
                    }
                });
            });
        });

        // Alarm settings
        ui.collapsing("Alarm Settings", |ui| {
            ALARM_VOLUME.with(|vol| {
                let mut volume = *vol.borrow();
                ui.horizontal(|ui| {
                    ui.label("Volume:");
                    ui.add(egui::Slider::new(&mut volume, 0.0..=1.0));
                    *vol.borrow_mut() = volume;
                });
            });

            ALARM_PATH.with(|path| {
                let mut alarm_path = path.borrow_mut();
                ui.horizontal(|ui| {
                    ui.label("Sound file:");
                    ui.text_edit_singleline(&mut *alarm_path);
                });
            });

            ui.horizontal(|ui| {
                if ui.button("Test Alarm").clicked() {
                    if play_alarm_sound() {
                        status.show("🔔 Testing alarm sound!");
                    } else {
                        status.show("⚠️ Failed to play alarm sound!");
                    }
                }

                // Only show stop button if alarm is currently playing
                let alarm_playing = ALARM_PLAYING.with(|playing| *playing.borrow());
                if alarm_playing {
                    if ui.button("Stop").clicked() {
                        stop_alarm_sound();
                        status.show("Alarm stopped");
                    }
                }
            });
        });
    }

    ui.separator();
    ui.add_space(10.0);

    // Today's study time
    let today_minutes = study_data.get_today_minutes();

    ui.vertical_centered(|ui| {
        ui.label("Today's total study time:");
        ui.label(format!(
            "{:.1} minutes ({:.1} hours)",
            today_minutes,
            today_minutes / 60.0
        ));
    });

    ui.add_space(20.0);

    // Debug tools
    if let Some(debug_message) = debug_tools.ui(ui, timer) {
        status.show(&debug_message);
    }

    // Status message
    status.render(ui);
}

// Helper function to start a break
fn start_break(minutes: u64, status: &mut StatusMessage) {
    let end_time = Instant::now() + Duration::from_secs(minutes * 60);

    BREAK_END_TIME.with(|break_end_time| {
        *break_end_time.borrow_mut() = Some(end_time);
    });

    status.show(&format!("Break started for {} minutes", minutes));
}

// Helper function to play the alarm sound
fn play_alarm_sound() -> bool {
    let volume = ALARM_VOLUME.with(|v| *v.borrow());
    let path = ALARM_PATH.with(|p| p.borrow().clone());

    // Stop any currently playing alarm first
    stop_alarm_sound();

    // Mark alarm as playing
    ALARM_PLAYING.with(|playing| {
        *playing.borrow_mut() = true;
    });

    #[cfg(target_os = "macos")]
    {
        // On macOS, use afplay with volume control
        match Command::new("afplay")
            .arg("-v")
            .arg(volume.to_string()) // Volume from 0 to 1
            .arg(&path)
            .spawn()
        {
            Ok(child) => {
                // Store the process handle so we can kill it later if needed
                AUDIO_PROCESS.with(|process| {
                    *process.borrow_mut() = Some(child);
                });
                return true;
            }
            Err(_) => return false,
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use PowerShell to play a sound (no volume control)
        match Command::new("powershell")
            .arg("-c")
            .arg(format!(
                "(New-Object Media.SoundPlayer '{}').PlaySync()",
                path
            ))
            .spawn()
        {
            Ok(child) => {
                // Store the process handle so we can kill it later if needed
                AUDIO_PROCESS.with(|process| {
                    *process.borrow_mut() = Some(child);
                });
                return true;
            }
            Err(_) => return false,
        }
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, try to use paplay with volume control
        match Command::new("paplay").arg(&path).spawn().or_else(|_| {
            // Fall back to aplay if paplay is not available
            Command::new("aplay").arg(&path).spawn()
        }) {
            Ok(child) => {
                // Store the process handle so we can kill it later if needed
                AUDIO_PROCESS.with(|process| {
                    *process.borrow_mut() = Some(child);
                });
                return true;
            }
            Err(_) => return false,
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        return false; // Unsupported platform
    }
}

// Helper function to stop any currently playing alarm sound
fn stop_alarm_sound() {
    AUDIO_PROCESS.with(|process| {
        if let Some(mut child) = process.borrow_mut().take() {
            // Try to kill the process
            let _ = child.kill();
        }
    });

    // Mark alarm as not playing
    ALARM_PLAYING.with(|playing| {
        *playing.borrow_mut() = false;
    });
}
