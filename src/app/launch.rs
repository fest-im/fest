use gio;
use gio::prelude::*;
use gtk;
use gtk::prelude::*;

/// Connect signals which are activated when the application is launched.
pub(super) fn connect(gtk_app: gtk::Application, gtk_builder: gtk::Builder) {
    gtk_app.connect_activate(clone!(gtk_builder => move |app| {
        // Add app actions
        let act_prefs = gio::SimpleAction::new("preferences", None);
        let act_shortcuts = gio::SimpleAction::new("shortcuts", None);
        let act_about = gio::SimpleAction::new("about", None);
        let act_quit = gio::SimpleAction::new("quit", None);

        act_quit.connect_activate(clone!(app => move |_, _| {
            app.quit();
        }));
        app.add_action(&act_quit);

        // Set up UI navigation button callbacks
        // It would be nice to use Popover::{popup,popdown} throughout here,
        // but that is only available in gtk 3.22.
        let rd_popover: gtk::Popover = gtk_builder.get_object("room_details_popover")
            .expect("Couldn't find room details popover in ui file.");
        let rd_stack: gtk::Stack = gtk_builder.get_object("rd_stack")
            .expect("Couldn't find room details stack in ui file.");
        let rp_revealer: gtk::Revealer = gtk_builder.get_object("right_pane_revealer")
            .expect("Couldn't find right pane revealer in ui file.");
        let rv_stack: gtk::Stack = gtk_builder.get_object("room_view_stack")
            .expect("Couldn't find room view stack in ui file.");

        rd_popover.connect_hide(clone!(rd_stack => move |_| {
            rd_stack.set_visible_child_name("details");
        }));

        let rd_invite_button: gtk::Button = gtk_builder.get_object("rd_invite_button")
            .expect("Couldn't find room invite button in ui file.");

        rd_invite_button.connect_clicked(clone!(rd_stack => move |_| {
            rd_stack.set_visible_child_name("invite");
        }));

        let rdi_cancel_button: gtk::Button = gtk_builder.get_object("rdi_cancel_button")
            .expect("Couldn't find room invite cancel button in ui file.");

        rdi_cancel_button.connect_clicked(clone!(rd_popover => move |_| {
            rd_popover.hide();
        }));

        let rd_leave_button: gtk::Button = gtk_builder.get_object("rd_leave_button")
            .expect("Couldn't find room leave button in ui file.");

        rd_leave_button.connect_clicked(clone!(rd_stack => move |_| {
            rd_stack.set_visible_child_name("leave");
        }));

        let rdl_stay_button: gtk::Button = gtk_builder.get_object("rdl_stay_button")
            .expect("Couldn't find room leave cancel button in ui file.");

        rdl_stay_button.connect_clicked(clone!(rd_popover => move |_| {
            rd_popover.hide();
        }));

        let rd_pins_toggle: gtk::ToggleButton = gtk_builder.get_object("rd_pins_button")
            .expect("Couldn't find room pins button in ui file.");

        rd_pins_toggle.connect_toggled(clone!(rd_popover => move |_toggle| {
            // TODO: Toggle room's pinned messages
            rd_popover.hide();
        }));

        let rp_toggle: gtk::ToggleButton = gtk_builder.get_object("right_pane_toggle")
            .expect("Couldn't find right pane toggle button.");

        rp_toggle.connect_toggled(clone!(rp_revealer => move |toggle| {
            rp_revealer.set_reveal_child(toggle.get_active())
        }));

        let rd_settings_toggle: gtk::ToggleButton = gtk_builder.get_object("rd_settings_button")
            .expect("Couldn't find room settings button in ui file.");

        rd_settings_toggle.connect_toggled(clone!(
            rd_pins_toggle,
            rd_popover,
            rp_toggle,
            rv_stack => move |toggle| {
                let active = toggle.get_active();
                rd_pins_toggle.set_visible(!active);
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


        let u_menu: gtk::PopoverMenu = gtk_builder.get_object("user_menu")
            .expect("Couldn't find user menu in ui file.");

        let u_register_button: gtk::Button = gtk_builder.get_object("u_register_button")
            .expect("Couldn't find user register button in ui file.");

        u_register_button.connect_clicked(clone!(u_menu => move |_| {
            u_menu.open_submenu("new_password");
        }));

        // When switching the main window stack child, we need to modify the
        // header bar to match
        let mw_stack: gtk::Stack = gtk_builder.get_object("main_window_stack")
            .expect("Couldn't find main window stack in ui file.");
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

        let view_switcher = clone!(
            h_accounts_button,
            h_back_button,
            h_bar,
            h_search_button,
            mw_stack,
            rp_toggle,
            title_button => move |view, title, subtitle, back| {
                let is_room = "room_view" == view;
                h_accounts_button.set_visible(is_room);
                h_search_button.set_visible(is_room);
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

        let lp_directory_button: gtk::Button = gtk_builder.get_object("lp_directory_button")
            .expect("Couldn't find directory button in ui file.");

        lp_directory_button.connect_clicked(clone!(view_switcher => move |_| {
            view_switcher("directory_view", "Directory", "", Some("Back"));
        }));

        h_back_button.connect_clicked(clone!(view_switcher => move |_| {
            // TODO: Set to actual room name
            view_switcher("room_view", "Fest", "", None);
        }));

        // Set up composer callbacks
        let ri_popover: gtk::Popover = gtk_builder.get_object("room_interactions_popover")
            .expect("Couldn't find room interactions popover in ui file.");

        let ri_attach_button: gtk::Button = gtk_builder.get_object("ri_attach_button")
            .expect("Couldn't find send attachment button in ui file.");

        ri_attach_button.connect_clicked(clone!(ri_popover => move |_| {
            // TODO: Open file chooser and send attachment
            ri_popover.hide();
        }));

        let ri_video_button: gtk::Button = gtk_builder.get_object("ri_video_button")
            .expect("Couldn't find video call button in ui file.");

        ri_video_button.connect_clicked(clone!(ri_popover => move |_| {
            // TODO: Start video call
            ri_popover.hide();
        }));

        let ri_voice_button: gtk::Button = gtk_builder.get_object("ri_voice_button")
            .expect("Couldn't find voice call button in ui file.");

        ri_voice_button.connect_clicked(clone!(ri_popover => move |_| {
            // TODO: Start voice call
            ri_popover.hide();
        }));

        // Setup greeter and related functions
        // TODO: Make this is only show on first run
        view_switcher("greeter_view", "Fest", "Matrix chat client", None);

        let gv_guest_button: gtk::Button = gtk_builder.get_object("gv_guest_button")
            .expect("Couldn't find greeter view guest button in ui file.");

        gv_guest_button.connect_clicked(clone!(view_switcher => move |_| {
            view_switcher("directory_view", "Directory", "", Some("Skip"));
        }));

        // Set up shutdown callback
        let window: gtk::Window = gtk_builder.get_object("main_window")
            .expect("Couldn't find main_window in ui file.");

        window.connect_delete_event(clone!(app => move |_, _| {
            app.quit();
            Inhibit(false)
        }));

        // Set up window configuration
        window.set_title("Fest");
        rv_stack.set_visible_child_name("chat");

        // Associate window with the Application and show it
        window.set_application(Some(app));
        window.present();
    }));
}
