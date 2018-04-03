use futures::{self, Sink};
use gio::{self, prelude::*};
use gtk::{self, prelude::*};

use crate::bg_thread::{MatrixCommand, UserSpecificCommand};

/// Connect signals which are activated when the application is launched.
pub(super) fn connect(
    gtk_app: gtk::Application,
    gtk_builder: gtk::Builder,
    backend_chan_tx: futures::sync::mpsc::Sender<MatrixCommand>,
) {
    gtk_app.connect_activate(clone!(gtk_builder, backend_chan_tx => move |app| {
        // Add app actions
        // TODO: Implement prefs, shortcuts, and about actions
        let _act_prefs = gio::SimpleAction::new("preferences", None);
        let act_shortcuts = gio::SimpleAction::new("shortcuts", None);
        let act_about = gio::SimpleAction::new("about", None);
        let act_quit = gio::SimpleAction::new("quit", None);

        act_quit.connect_activate(clone!(app => move |_, _| {
            app.quit();
        }));
        app.add_action(&act_quit);

        // Set up UI navigation button callbacks and relevant window actions
        // It would be nice to use Popover::{popup,popdown} throughout here,
        // but that is only available in gtk 3.22.
        let mw_stack: gtk::Stack = gtk_builder.get_object("main_window_stack")
            .expect("Couldn't find main window stack in ui file.");
        let rd_popover: gtk::Popover = gtk_builder.get_object("room_details_popover")
            .expect("Couldn't find room details popover in ui file.");
        let rd_stack: gtk::Stack = gtk_builder.get_object("rd_stack")
            .expect("Couldn't find room details stack in ui file.");
        let rp_revealer: gtk::Revealer = gtk_builder.get_object("right_pane_revealer")
            .expect("Couldn't find right pane revealer in ui file.");
        let rv_stack: gtk::Stack = gtk_builder.get_object("room_view_stack")
            .expect("Couldn't find room view stack in ui file.");
        let window: gtk::ApplicationWindow = gtk_builder.get_object("main_window")
            .expect("Couldn't find main_window in ui file.");

        // This shortcut window is for backwards compat with gtk 3.16,
        // when new version is used (>= 3.20), it can be
        // replaced with ```win.show-help-overlay``` in action of
        // Keyboard shortcuts in menus.ui

        // Parts mentioning shortcuts here can be then removed

        act_shortcuts.connect_activate(clone!(app => move |_, _| {
            let dialog: gtk::Window = gtk::Builder::new_from_resource("/org/fest-im/fest/gtk/help-overlay-old.ui")
                .get_object("help_overlay_old")
                .expect("Couldn't find help_overlay_old in ui file.");

            dialog.show();
        }));
        act_about.connect_activate(clone!(window => move |_, _| {
            let dialog = gtk::AboutDialog::new();

            dialog.set_modal(true);
            dialog.set_transient_for(&window);
            dialog.set_logo_icon_name("fest");
            dialog.set_program_name("Fest");
            dialog.set_version(env!("CARGO_PKG_VERSION"));
            dialog.set_website_label("Contribute to Fest");
            dialog.set_website("https://github.com/fest-im/fest");
            dialog.set_license_type(gtk::License::Gpl30);

            dialog.set_artists(&[
                "Stasiek Michalski <hellcp@opensuse.org>",
            ]);

            dialog.set_authors(&[
                "Andrew Conrad",
                "Jonas Platte",
            ]);

            dialog.show();
        }));

        app.add_action(&act_shortcuts);
        app.add_action(&act_about);

        // Reset certain widgets to their default state when hidden
        rd_popover.connect_hide(clone!(rd_stack => move |_| {
            rd_stack.set_visible_child_name("details");
        }));

        mw_stack.connect_property_visible_child_name_notify(clone!(rv_stack => move |stack| {
            if stack.get_visible_child_name() != Some("room".to_string()) {
                rv_stack.set_visible_child_name("chat");
            }
        }));

        // Set up popover for inviting people to a room
        let act_show_rd_invite = gio::SimpleAction::new("show_rd_invite", None);
        let rd_invite_button: gtk::Button = gtk_builder.get_object("rd_invite_button")
            .expect("Couldn't find room invite button in ui file.");

        act_show_rd_invite.connect_activate(clone!(rd_popover, rd_stack => move |_, _| {
            rd_stack.set_visible_child_name("invite");
            if !rd_popover.is_visible() {
                rd_popover.show();
            }
        }));
        window.add_action(&act_show_rd_invite);

        rd_invite_button.connect_clicked(clone!(act_show_rd_invite => move |_| {
            act_show_rd_invite.activate(None);
        }));

        let rdi_cancel_button: gtk::Button = gtk_builder.get_object("rdi_cancel_button")
            .expect("Couldn't find room invite cancel button in ui file.");

        rdi_cancel_button.connect_clicked(clone!(rd_popover => move |_| {
            rd_popover.hide();
        }));

        // Set up popover for leaving a room
        let act_show_rd_leave = gio::SimpleAction::new("show_rd_leave", None);
        let rd_leave_button: gtk::Button = gtk_builder.get_object("rd_leave_button")
            .expect("Couldn't find room leave button in ui file.");

        act_show_rd_leave.connect_activate(clone!(rd_popover, rd_stack => move |_, _| {
            rd_stack.set_visible_child_name("leave");
            if !rd_popover.is_visible() {
                rd_popover.show();
            }
        }));
        window.add_action(&act_show_rd_leave);

        rd_leave_button.connect_clicked(clone!(act_show_rd_leave => move |_| {
            act_show_rd_leave.activate(None);
        }));

        let rdl_stay_button: gtk::Button = gtk_builder.get_object("rdl_stay_button")
            .expect("Couldn't find room leave cancel button in ui file.");

        rdl_stay_button.connect_clicked(clone!(rd_popover => move |_| {
            rd_popover.hide();
        }));

        // Set up room pins toggle
        let act_toggle_room_pins = gio::SimpleAction::new("toggle_room_pins", None);
        let pins_revealer: gtk::Revealer = gtk_builder.get_object("pins_revealer")
            .expect("Couldn't find pins revealer in ui file.");
        let rd_pins_toggle: gtk::ToggleButton = gtk_builder.get_object("rd_pins_button")
            .expect("Couldn't find room pins button in ui file.");

        act_toggle_room_pins.connect_activate(clone!(rd_pins_toggle => move |_, _| {
            rd_pins_toggle.clicked();
        }));
        window.add_action(&act_toggle_room_pins);

        rd_pins_toggle.connect_toggled(clone!(pins_revealer, rd_popover => move |toggle| {
            pins_revealer.set_reveal_child(toggle.get_active());
            rd_popover.hide();
        }));

        // Set up right panel toggle
        let act_toggle_right_pane = gio::SimpleAction::new("toggle_right_pane", None);
        let rp_toggle: gtk::ToggleButton = gtk_builder.get_object("right_pane_toggle")
            .expect("Couldn't find right pane toggle button.");

        act_toggle_right_pane.connect_activate(clone!(rp_toggle => move |_, _| {
            rp_toggle.clicked();
        }));
        window.add_action(&act_toggle_right_pane);

        rp_toggle.connect_toggled(clone!(rp_revealer => move |toggle| {
            rp_revealer.set_reveal_child(toggle.get_active())
        }));

        // Set up room settings view
        let act_toggle_room_settings = gio::SimpleAction::new("toggle_room_settings", None);
        let rd_settings_toggle: gtk::ToggleButton = gtk_builder.get_object("rd_settings_button")
            .expect("Couldn't find room settings button in ui file.");

        act_toggle_room_settings.connect_activate(clone!(rd_settings_toggle => move |_, _| {
            rd_settings_toggle.clicked();
        }));

        rd_settings_toggle.connect_toggled(clone!(
            act_toggle_right_pane,
            act_toggle_room_pins,
            rd_pins_toggle,
            rd_popover,
            rp_toggle,
            rv_stack => move |toggle| {
                let active = toggle.get_active();
                act_toggle_room_pins.set_enabled(!active);
                rd_pins_toggle.set_visible(!active);
                act_toggle_right_pane.set_enabled(!active);
                rp_toggle.set_visible(!active);
                rd_popover.hide();

                if active {
                    rv_stack.set_visible_child_name("settings");
                } else {
                    rv_stack.set_visible_child_name("chat");
                }
            }
        ));

        let rvs_back_button: gtk::Button = gtk_builder.get_object("rvs_back_button")
            .expect("Couldn't find room settings back button in ui file.");

        rvs_back_button.connect_clicked(clone!(rd_settings_toggle => move |_| {
            rd_settings_toggle.clicked();
        }));

        // Set up user menu
        let act_show_user_menu = gio::SimpleAction::new("show_user_menu", None);
        let u_menu: gtk::PopoverMenu = gtk_builder.get_object("user_menu")
            .expect("Couldn't find user menu in ui file.");
        let u_register_button: gtk::Button = gtk_builder.get_object("u_register_button")
            .expect("Couldn't find user register button in ui file.");

        act_show_user_menu.connect_activate(clone!(u_menu => move |_, _| {
            u_menu.show();
        }));
        window.add_action(&act_show_user_menu);

        u_register_button.connect_clicked(clone!(u_menu => move |_| {
            u_menu.open_submenu("new_password");
        }));

        // When switching the main window stack child, we need to modify the
        // header bar to match
        let h_bar: gtk::HeaderBar = gtk_builder.get_object("header_bar")
            .expect("Couldn't find header bar in ui file.");
        let h_accounts_button: gtk::Button = gtk_builder.get_object("header_accounts_button")
            .expect("Couldn't find header accounts button in ui file.");
        let h_back_button: gtk::Button = gtk_builder.get_object("header_back_button")
            .expect("Couldn't find header back button in ui file.");
        let title_button: gtk::Widget = gtk_builder.get_object("title_menu_button")
            .expect("Couldn't find room title menu button in ui file.");
        let h_search_button: gtk::Button = gtk_builder.get_object("header_search_button")
            .expect("Couldn't find header search button in ui file.");

        // TODO: Disable room and directory vew switch actions when there is not
        // an account set up yet.
        let view_switcher = clone!(
            act_show_user_menu,
            act_toggle_right_pane,
            h_accounts_button,
            h_back_button,
            h_bar,
            h_search_button,
            mw_stack,
            rp_toggle,
            title_button => move |view, title, subtitle, back| {
                let is_room = "room_view" == view;
                act_show_user_menu.set_enabled(is_room);
                h_accounts_button.set_visible(is_room);
                h_search_button.set_visible(is_room);
                act_toggle_right_pane.set_enabled(is_room);
                rp_toggle.set_visible(is_room);

                if is_room {
                    h_back_button.hide();

                    h_bar.set_property_custom_title(Some(&title_button));
                } else {
                    if let Some(s) = back {
                        h_back_button.set_label(s);
                        h_back_button.show();
                    } else {
                        h_back_button.hide();
                    }

                    h_bar.set_property_custom_title::<gtk::Widget>(None);
                }

                h_bar.set_title(title);
                h_bar.set_subtitle(subtitle);

                mw_stack.set_visible_child_name(view);
            }
        );

        // Set up directory view
        let act_show_dir_view = gio::SimpleAction::new("show_dir_view", None);
        let lp_directory_button: gtk::Button = gtk_builder.get_object("lp_directory_button")
            .expect("Couldn't find directory button in ui file.");

        act_show_dir_view.connect_activate(clone!(view_switcher, backend_chan_tx => move |_, _| {
            view_switcher("directory_view", "Directory", "", Some("Back"));
            // TODO: Do we want to handle send errors?
            // TODO: Replace 0, it is just a dummy User ID
            let _ = backend_chan_tx.clone().wait().send(MatrixCommand::UserSpecificCommand {
                user_id: 0,
                command: UserSpecificCommand::FetchDirectory,
            });
        }));
        window.add_action(&act_show_dir_view);

        lp_directory_button.connect_clicked(clone!(act_show_dir_view => move |_| {
            act_show_dir_view.activate(None);
        }));

        // Set up room view
        let act_show_room_view = gio::SimpleAction::new("show_room_view", None);

        act_show_room_view.connect_activate(clone!(view_switcher => move |_, _| {
            // TODO: Set to actual room name
            view_switcher("room_view", "Fest", "", None);
        }));
        window.add_action(&act_show_room_view);

        h_back_button.connect_clicked(clone!(act_show_room_view => move |_| {
            act_show_room_view.activate(None);
        }));

        // Set up composer callbacks
        let ri_popover: gtk::Popover = gtk_builder.get_object("room_interactions_popover")
            .expect("Couldn't find room interactions popover in ui file.");

        // Set up file attachment
        let act_attach_file = gio::SimpleAction::new("attach_file", None);
        let ri_attach_button: gtk::Button = gtk_builder.get_object("ri_attach_button")
            .expect("Couldn't find send attachment button in ui file.");

        act_attach_file.connect_activate(clone!(ri_popover => move |_, _| {
            // TODO: Open file chooser and send attachment
            ri_popover.hide();
        }));
        window.add_action(&act_attach_file);

        ri_attach_button.connect_clicked(clone!(act_attach_file => move |_| {
            act_attach_file.activate(None);
        }));

        // Set up video call
        let act_video_call = gio::SimpleAction::new("video_call", None);
        let ri_video_button: gtk::Button = gtk_builder.get_object("ri_video_button")
            .expect("Couldn't find video call button in ui file.");

        act_video_call.connect_activate(clone!(ri_popover => move |_, _| {
            ri_popover.hide();
        }));
        window.add_action(&act_video_call);

        ri_video_button.connect_clicked(clone!(act_video_call => move |_| {
            act_video_call.activate(None);
        }));

        // Set up voice call
        let act_voice_call = gio::SimpleAction::new("voice_call", None);
        let ri_voice_button: gtk::Button = gtk_builder.get_object("ri_voice_button")
            .expect("Couldn't find voice call button in ui file.");

        act_voice_call.connect_activate(clone!(ri_popover => move |_, _| {
            // TODO: Start voice call
            ri_popover.hide();
        }));
        window.add_action(&act_voice_call);

        ri_voice_button.connect_clicked(clone!(act_voice_call => move |_| {
            act_voice_call.activate(None);
        }));

        // Set up markdown formatting toggling and notification
        let act_toggle_markdown = gio::SimpleAction::new("toggle_markdown", None);
        let _composer_entry: gtk::Entry = gtk_builder.get_object("composer_entry")
            .expect("Couldn't find composer entry in ui file.");
        let rvc_notif_revealer: gtk::Revealer = gtk_builder.get_object("rvc_notif_revealer")
            .expect("Couldn't find chat notification revealer in ui file.");
        let rvc_notif_label: gtk::Label = gtk_builder.get_object("rvc_notif_label")
            .expect("Couldn't find chat notification label in ui file.");
        let _rvc_notif_undo_button: gtk::Button = gtk_builder.get_object("rvc_notif_undo_button")
            .expect("Couldn't find chat notification undo button in ui file.");
        let rvc_notif_close_button: gtk::Button = gtk_builder.get_object("rvc_notif_close_button")
            .expect("Couldn't find chat notification close button in ui file.");

        act_toggle_markdown.connect_activate(clone!(
            rvc_notif_label,
            rvc_notif_revealer => move |_, _| {
                // TODO: Toggle formatting and save with gsettings. Store new
                // value in markdown_enabled.
                let markdown_enabled = true;
                let s = "Markdown formatting has been ";
                let msg: String;

                if markdown_enabled {
                    msg = [s, "enabled."].join("");
                } else {
                    msg = [s, "disabled."].join("");
                }

                // TODO: Automatically dismiss the notification after a timeout?

                rvc_notif_label.set_text(&msg);
                rvc_notif_revealer.set_reveal_child(true);
            }
        ));
        app.add_action(&act_toggle_markdown);

        // TODO: Only activate accelerator when composer_entry is focused?
        app.set_accels_for_action("app.toggle_markdown", &["<Ctl>m"]);
        // TODO: Set up undo

        rvc_notif_close_button.connect_clicked(clone!(rvc_notif_revealer => move |_| {
            rvc_notif_revealer.set_reveal_child(false);
        }));

        // Set up greeter and related functions
        // TODO: Make this is only show on first run
        view_switcher("greeter_view", "Fest", "Matrix chat client", None);

        let gv_guest_button: gtk::Button = gtk_builder.get_object("gv_guest_button")
            .expect("Couldn't find greeter view guest button in ui file.");

        gv_guest_button.connect_clicked(clone!(view_switcher => move |_| {
            view_switcher("directory_view", "Directory", "", Some("Skip"));
        }));

        // Set up action accelerators
        app.set_accels_for_action("app.quit", &["<Ctl>q"]);
        app.set_accels_for_action("win.show_rd_invite", &["<Ctl>i"]);
        app.set_accels_for_action("win.show_rd_leave", &["<Ctl>l"]);
        app.set_accels_for_action("win.toggle_room_pins", &["F7"]);
        app.set_accels_for_action("win.toggle_right_pane", &["F6"]);
        app.set_accels_for_action("win.show_user_menu", &["<Ctl>u"]);
        app.set_accels_for_action("win.show_dir_view", &["F3"]);
        app.set_accels_for_action("win.show_room_view", &["F2"]);
        app.set_accels_for_action("win.attach_file", &["<Ctl><Shift>a"]);
        app.set_accels_for_action("win.video_call", &["<Ctl><Shift>v"]);
        app.set_accels_for_action("win.voice_call", &["<Ctl><Shift>c"]);

        // Set up shutdown callback
        window.connect_delete_event(clone!(act_quit => move |_, _| {
            act_quit.activate(None);
            Inhibit(false)
        }));

        // Set up window configuration
        window.set_title("Fest");
        window.set_icon_name("fest");

        // Associate window with the Application and show it
        window.set_application(Some(app));
        window.present();
    }));
}
