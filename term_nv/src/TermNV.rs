use std::ffi::{CStr, CString};

use gdk::*;
use glib::ObjectExt;

use gtk::prelude::*;
use gtk::*;
use gtk::{ApplicationWindow, ButtonsType, HeaderBar, MessageDialog, MessageType};

use once_cell::sync::Lazy;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use glib::translate::*;
use libc::{EXIT_FAILURE, EXIT_SUCCESS, pid_t};
use std::env;

const TERMINAL_COLS: i64 = 127;
const TERMINAL_ROWS: i64 = 42;

//==================================================================
//gestion des programme autorisé
//==================================================================

// Chemin fixe vers la bibliothèque de programmes
const PGM_LIB_DIR: &str = "/usr/bin/";

const WORPGM: &str = "nvim";

// Liste des programmes autorisés (clé LDA)
const AUTHORIZED_PROGRAMS: &[&str] = &["nvim"];

// Vérifie si le programme est autorisé
fn is_authorized_program(program_name: &str) -> bool {
    AUTHORIZED_PROGRAMS.contains(&program_name)
}

// Construit le chemin complet vers le programme
fn get_program_path(program_name: &str) -> &'static str {
    let s = format!("{}{}", PGM_LIB_DIR, program_name);
    std::boxed::Box::leak(s.into_boxed_str())
}

//===============================================================

//================================
// gestion ALT_F4 et log_message

// Pointeurs atomiques globaux pour la fenêtre et le terminal
static WINDOW_PTR: AtomicPtr<gtk_sys::GtkWindow> = AtomicPtr::new(ptr::null_mut());
// Variable globale thread-safe
pub static STATE_CLOSE: AtomicBool = AtomicBool::new(false); // continue

// Fonction pour gérer l'appui sur Alt+F4
static ALTF4: Lazy<bool> = Lazy::new(|| true); // ou false

fn key_press_altf4(window: &ApplicationWindow) -> bool {
    let dialog = MessageDialog::new(
        Some(window),
        gtk::DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        "Voulez-vous vraiment quitter ?", // Remplace par MESSAGE_ALT_F4
    );

    let response = dialog.run();
    unsafe {
        dialog.destroy();
    } // Corrigé : bloc unsafe avec accolades

    match response {
        gtk::ResponseType::Yes => {
            std::process::exit(EXIT_FAILURE);
        }
        _ => {
            true // Équivalent à GDK_EVENT_STOP
        }
    }
}

// l'application ce termine correctement
fn win_close(_window: &ApplicationWindow) -> bool {
    let quit = STATE_CLOSE.load(Ordering::SeqCst);

    if quit {
        std::process::exit(EXIT_SUCCESS);
    };
    true
}

//une fonction pour debug
// use std::io::Write; // Requis pour utiliser writeln! sur un fichier
// fn log_message(msg: &str) {
//     let timestamp = chrono::Local::now().format("%H:%M:%S");
//
//     // 1. Affichage dans la console
//     println!("[{}] {}", timestamp, msg);
//
//     // 2. Écriture dans le fichier "terminal.log" du répertoire courant
//     // .create(true) : crée le fichier s'il n'existe pas
//     // .append(true) : ajoute le texte à la fin du fichier sans l'écraser
//     if let Ok(mut file) = std::fs::OpenOptions::new()
//         .create(true)
//         .append(true)
//         .open("terminal.log")
//     {
//         // Écrit la ligne dans le fichier
//         let _ = writeln!(file, "[{}] {}", timestamp, msg);
//     }
// }

//============================================
// gestion du terminal
//============================================

// 2 arguments
// le titre de la fenetre Nom du projet etc
// la bibliothèque  dans la quelle on travail

fn main() {
    let args: Vec<String> = env::args().collect();

    // Déterminer le programme à exécuter
    let program_name = WORPGM;

    // Répertoire de travail
    let wrkdir = CString::new(args[2].clone()).expect("Répertoire invalide");

    if !is_authorized_program(program_name) {
        eprintln!("Programme non autorisé : {}", program_name);
        std::process::exit(EXIT_FAILURE);
    }

    // Construire les arguments de la commande

    // log_message(&format!("args.len() {:?}\n", args.len()));

    let command = CString::new(get_program_path(program_name)).unwrap();
    let mut command_args = vec![command.into_raw(), ptr::null_mut()];

    // log_message(&format!("Contenu de command_args: {:?}\n", args_content));

    // Construire les variables d'environnement
    let current_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", current_path, args[2].clone());
    let env_term = CString::new("TERM=xterm-256color").expect("TERM invalide");
    let env_path = CString::new(format!("PATH={}", new_path)).expect("PATH invalide");

    let mut envp_args: Vec<*mut libc::c_char> = vec![
        env_term.into_raw(),
        env_path.into_raw(),
        ptr::null_mut(), // Terminer par NULL
    ];

    // Variable pour stocker le PID de l'enfant
    let mut child_pid: pid_t = 0;

    //===================================================
    //construire le terminal
    //===================================================

    // Initialiser GTK
    gtk::init().expect("Échec de l'initialisation de GTK");

    // Créer une fenêtre GTK
    // le titre vas être determiner par l'applicattion du terminal
    let window = gtk::ApplicationWindow::builder().title(args[1].clone()).build();

    // active le redimensionnement de la fenêtre
    window.set_resizable(true);

    // Contrôler la possibilité de fermer la fenêtre
    window.set_deletable(*ALTF4);

    // Dans ton code principal, après avoir créé la fenêtre :
    // Dans ton code principal, après avoir créé la fenêtre :
    if *ALTF4 {
        // mode développeur
        window.connect_delete_event(|window, _| key_press_altf4(&window.clone()).into());
    } else {
        //envirronement programeur TEST
        let header_bar = HeaderBar::new();

        header_bar.set_decoration_layout(None);
        window.set_titlebar(Some(&header_bar));

        // uniquement si l'application terminal est close
        window.connect_delete_event(|window, _| win_close(&window.clone()).into());
    }

    unsafe {
        // Stocker les pointeurs globaux
        WINDOW_PTR.store(window.as_ptr() as *mut gtk_sys::GtkWindow, Ordering::SeqCst);
        let window_ptr = WINDOW_PTR.load(Ordering::SeqCst);

        // // Récupérer le GdkWindow associé au GtkWindow
        // let gdk_window = gtk_sys::gtk_widget_get_window(window_ptr as *mut gtk_sys::GtkWidget);
        // gdk_sys::gdk_window_move(gdk_window, 10 as i32, 10 as i32);

        let terminal = vte_sys::vte_terminal_new();

        // 1. Configurer la police avec le bon nom
        let font_desc = pango_sys::pango_font_description_from_string(
            CString::new("FiraCode Nerd Font Regular 14").unwrap().as_ptr(),
        );
        vte_sys::vte_terminal_set_font(terminal, font_desc as *const _);

        vte_sys::vte_terminal_set_size(terminal, TERMINAL_COLS, TERMINAL_ROWS);
        vte_sys::vte_terminal_set_scrollback_lines(terminal, 0);
        vte_sys::vte_terminal_set_scroll_on_output(terminal, 0);
        vte_sys::vte_terminal_set_scroll_on_keystroke(terminal, 0);
        vte_sys::vte_terminal_set_mouse_autohide(terminal, 1);
        vte_sys::vte_terminal_set_cursor_blink_mode(terminal, vte_sys::VTE_CURSOR_BLINK_ON);
        vte_sys::vte_terminal_set_cursor_shape(terminal, vte_sys::VTE_CURSOR_SHAPE_BLOCK);

        // // Lancer la commande
        // let wrkdir = CString::new(PGM_LIB_DIR).unwrap();
        // let command = CString::new("/home/soleil/Zrust/gen_sda/gensda").unwrap();
        // let mut args: Vec<*mut libc::c_char> = vec![command.into_raw(), ptr::null_mut()];
        // let mut child_pid: pid_t = 0;

        let spawn_result = vte_sys::vte_terminal_spawn_sync(
            terminal,
            vte_sys::VTE_PTY_DEFAULT,
            wrkdir.as_ptr(),
            command_args.as_mut_ptr(),
            envp_args.as_mut_ptr(),
            glib_sys::G_SPAWN_SEARCH_PATH | glib_sys::G_SPAWN_FILE_AND_ARGV_ZERO,
            None,
            ptr::null_mut(),
            &mut child_pid,
            ptr::null_mut(),
            ptr::null_mut(),
        );

        if spawn_result == 0 {
            eprintln!("Erreur lors du spawn du processus dans le terminal");
        }

        // Libérer les arguments de la commande
        for &arg in &command_args {
            if !arg.is_null() {
                drop(CString::from_raw(arg)); // Libère la mémoire explicitement
            }
        }
        // Libérer les variables d'environnement
        for &arg in &envp_args {
            if !arg.is_null() {
                drop(CString::from_raw(arg)); // Libère la mémoire explicitement
            }
        }

        // Convertir le terminal en widget GTK
        let terminal_widget = gtk::Widget::from_glib_none(terminal as *mut gtk_sys::GtkWidget);
        window.add(&terminal_widget);

        // Convertir le pointeur brut en un objet glib::Object
        let terminal_obj = glib::Object::from_glib_none(terminal as *mut _);

        // Connecter le signal "resize-window" avec connect_unsafe
        let _handler_id_resize = terminal_obj.connect_unsafe("resize-window", false, |args| {
            let cols = args[1].get::<u32>().unwrap();
            let rows = args[2].get::<u32>().unwrap();

            // Récupérer le terminal depuis args[0]
            let width = vte_sys::vte_terminal_get_char_width(terminal);
            let height = vte_sys::vte_terminal_get_char_height(terminal);

            if width > 0 && height > 0 {
                gtk_sys::gtk_window_resize(
                    window_ptr,
                    ((width * (cols + 1) as i64) - 9) as i32,
                    ((height * (rows + 1) as i64) - 10) as i32,
                );
            } else {
                eprintln!("Erreur : width ou height est <= 0");
            }

            None
        });

        // Connecter le signal "window-title-changed" avec connect_unsafe
        let _handler_id_title = terminal_obj.connect_unsafe("window-title-changed", false, move |_| {
            let title_ptr = vte_sys::vte_terminal_get_window_title(terminal);
            if !title_ptr.is_null() {
                let title = CStr::from_ptr(title_ptr).to_string_lossy();
                let window = gtk::Window::from_glib_none(window_ptr);
                window.set_title(&title);
            }
            None
        });

        let _close_child = terminal_obj.connect_unsafe("child-exited", false, |_| {

                std::process::exit(EXIT_SUCCESS);

        });

        let _close_terminal = terminal_obj.connect_unsafe("destroy", false, |_| {
            STATE_CLOSE.store(true, Ordering::SeqCst);

            window.close();

            None
        });
    };

    // permet de déplacer et de rester static pour l'application
    window.set_gravity(Gravity::Static);
    window.show_all();

    // Lancer la boucle principale GTK
    gtk::main();
}
