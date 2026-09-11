use std::ffi::{CStr, CString};

use gtk::ApplicationWindow;
use gtk::prelude::*;
use gtk::*;

use std::ptr;

use glib::object::ObjectExt;
use glib::translate::*;
use libc::{EXIT_FAILURE, EXIT_SUCCESS};
use once_cell::sync::Lazy;
use std::env;

use std::io::Write; // Requis pour utiliser writeln! sur un fichier

const TERMINAL_COLS: i64 = 127;
const TERMINAL_ROWS: i64 = 42;

//==================================================================
//gestion des programme autorisé
//==================================================================

// Chemin fixe vers la bibliothèque de programmes
const PGM_LIB_DIR: &str = "/usr/bin/";

const WORPGM: &str = "nvim";

const GRAVITY_STATIC: i32 = 10;

// Liste des programmes autorisés (clé LDA)
const AUTHORIZED_PROGRAMS: &[&str] = &["nvim"];

// Vérifie si le programme est autorisé
fn is_authorized_program(program_name: &str) -> bool {
	AUTHORIZED_PROGRAMS.contains(&program_name)
}

// Construit le chemin complet vers le programme
fn get_program_path(program_name: &str) -> &'static str {
	let s = format!("{}{}",  PGM_LIB_DIR, program_name); // spéciphic nvim
	std::boxed::Box::leak(s.into_boxed_str())
}

fn get_path(lib_src: &str) -> &'static str {
	let s = format!("{}", lib_src); // spéciphic nvim
	std::boxed::Box::leak(s.into_boxed_str())
}

//===============================================================
//une fonction pour debug

fn log_message(msg: &str) {
	let timestamp = chrono::Local::now().format("%H:%M:%S");

	// 1. Affichage dans la console
	println!("[{}] {}", timestamp, msg);

	// 2. Écriture dans le fichier "terminal.log" du répertoire courant
	// .create(true) : crée le fichier s'il n'existe pas
	// .append(true) : ajoute le texte à la fin du fichier sans l'écraser
	if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("terminal.log") {
		// Écrit la ligne dans le fichier
		let _ = writeln!(file, "[{}] {}", timestamp, msg);
	}
}
//================================
/// Affiche une boîte de dialogue d'erreur bloquante (Modal)
fn afficher_erreur_fatale(titre: &str, message: &str) {
	// Crée une boîte de message GTK de type Erreur avec un bouton OK
	let dialog = gtk::MessageDialog::new(
		None::<&gtk::Window>,	 // Pas de fenêtre parente car l'application n'a pas démarré
		gtk::DialogFlags::MODAL, // Bloque l'application tant qu'on n'a pas cliqué
		gtk::MessageType::Error, // Style d'erreur rouge
		gtk::ButtonsType::Ok,	 // Bouton de fermeture unique
		titre,
	);

	// Ajoute le texte explicitatif détaillé
	dialog.set_secondary_text(Some(message));

	// Exécute la boîte de dialogue en mode bloquant et attend le clic
	dialog.run();

	// Détruit proprement le widget avant de quitter
	unsafe {
		dialog.destroy();
	}
}

// l'application c'est mal terminer
fn terminal_error(window: &ApplicationWindow) {
	let dialog = MessageDialog::new(
		Some(window),
		gtk::DialogFlags::MODAL,
		MessageType::Error,				   // ou MessageType::Info si tu préfères
		ButtonsType::Ok,				   // ✅ Un seul bouton "OK"
		"Veuillez consulter les logs SVP", // Message
	);

	dialog.run();

	unsafe {
		dialog.destroy(); // Ferme la boîte de dialogue
	}
}

// Fonction pour gérer l'appui sur Alt+F4
static ALTF4: Lazy<bool> = Lazy::new(|| true); // true for dev

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
		_ => true, // Équivalent à GDK_EVENT_STOP
	}
}
//============================================
// gestion du terminal
//============================================

// 2 arguments
// le titre de la fenetre Nom du projet etc
// la bibliothèque	dans la quelle on travail

fn main() {
	// 1. Initialisation obligatoire de GTK dès le début
	if gtk::init().is_err() {
		eprintln!("Impossible d'initialiser GTK.");
		std::process::exit(1);
	}

	let args: Vec<String> = env::args().collect();

	// Déterminer le programme à exécuter
	let program_name = WORPGM;

	// Répertoire de travail
	if args.len() < 2 {
		// Affiche la boîte de message d'erreur et quitte
		afficher_erreur_fatale("Paramètre manquant", "Répertoire invalide.");
		std::process::exit(1);
	};
	let wrkdir	= CString::new(get_path(&args[2] )).unwrap();

	if !is_authorized_program(program_name) {
		afficher_erreur_fatale("erreur fatal", &format!("Programme non autorisé : {}", program_name));
		std::process::exit(EXIT_FAILURE);
	}

	// Construire les arguments de la commande

	// log_message(&format!("args.len() {:?}\n", args.len()));

	let command = CString::new(get_program_path(&program_name)).unwrap();
	let mut command_args = vec![command.clone().into_raw(), ptr::null_mut()];


	// Construire les variables d'environnement
	let current_path = env::var("PATH").unwrap_or_default();
	let new_path = format!("{}:{}", current_path, get_path(&args[2] ));

	let env_term = CString::new("TERM=xterm-256color").expect("TERM invalide");
	let env_path = CString::new(format!("PATH={}", new_path)).expect("PATH invalide");

	let mut envp_args: Vec<*mut libc::c_char> = vec![
		env_term.into_raw(),
		env_path.into_raw(),
		ptr::null_mut(), // Terminer par NULL
	];

	//===================================================
	//construire le terminal
	//===================================================

	// Créer une fenêtre GTK
	// le titre vas être determiner par l'applicattion du terminal
	let window = gtk::ApplicationWindow::builder().title(args[1].clone()).build();
	let window_clone = window.clone();
	// active le redimensionnement de la fenêtre
	window.set_resizable(true);

	// Contrôler la possibilité de fermer la fenêtre
	window.set_deletable(*ALTF4);

	// Dans ton code principal, après avoir créé la fenêtre :
	// mode développeur
	// Dans ton code principal, après avoir créé la fenêtre :
	if *ALTF4 {
		// mode développeur
		window.connect_delete_event(|window, _| key_press_altf4(&window.clone()).into());
	}

	unsafe {
		let terminal = vte_sys::vte_terminal_new();

		// 1. Configurer la police avec le bon nom
		let font_desc = pango::FontDescription::from_string("FiraCode Nerd Font Regular 14");

		// 2. On extrait le pointeur avec son type FFI interne exact, puis on le cast pour vte_sys
		let raw_ptr: *const pango::ffi::PangoFontDescription = font_desc.to_glib_none().0;
		vte_sys::vte_terminal_set_font(terminal, raw_ptr as *const _);

		vte_sys::vte_terminal_set_size(terminal, TERMINAL_COLS, TERMINAL_ROWS);
		vte_sys::vte_terminal_set_scrollback_lines(terminal, 0);
		vte_sys::vte_terminal_set_scroll_on_output(terminal, 0);
		vte_sys::vte_terminal_set_scroll_on_keystroke(terminal, 0);
		vte_sys::vte_terminal_set_mouse_autohide(terminal, 1);
		vte_sys::vte_terminal_set_cursor_blink_mode(terminal, vte_sys::VTE_CURSOR_BLINK_ON);
		vte_sys::vte_terminal_set_cursor_shape(terminal, vte_sys::VTE_CURSOR_SHAPE_BLOCK);

		// Variable pour stocker le PID de l'enfant
		let mut child_pid_raw: std::os::raw::c_int = 0;
		let raw_flags = glib::SpawnFlags::SEARCH_PATH | glib::SpawnFlags::FILE_AND_ARGV_ZERO;
		let spawn_result = vte_sys::vte_terminal_spawn_sync(
			terminal,
			vte_sys::VTE_PTY_DEFAULT,
			wrkdir.as_ptr(),
			command_args.as_mut_ptr(),
			envp_args.as_mut_ptr(),
			raw_flags.into_glib() as u32,
			None,
			ptr::null_mut(),
			&mut child_pid_raw,
			ptr::null_mut(),
			ptr::null_mut(),
		);

		let window_app = window_clone.clone();
		if spawn_result == glib::ffi::GFALSE {
			log_message("Erreur critique : Impossible de démarrer le terminal VTE (vte_terminal_spawn_sync a échoué).");

			// Optionnel : Afficher l'erreur graphique immédiatement car la closure ne s'exécutera jamais
			terminal_error(&window_app);
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
		let terminal_widget: gtk::Widget = from_glib_none(terminal as *mut gtk::ffi::GtkWidget);

		window.add(&terminal_widget);

		// Convertir le pointeur brut en un objet glib::Object
		let terminal_obj = glib::Object::from_glib_none(terminal as *mut _);

		// Connecter le signal "resize-window" avec connect_unsafe
		let _handler_id_resize = terminal_widget.connect_unsafe("resize-window", false, |args| {
			let cols = args[1].get::<u32>().unwrap();
			let rows = args[2].get::<u32>().unwrap();

			// Récupérer le terminal depuis args[0]
			let width = vte_sys::vte_terminal_get_char_width(terminal);
			let height = vte_sys::vte_terminal_get_char_height(terminal);

			if width > 0 && height > 0 {
				window.resize(((width * (cols + 1) as i64) - 9) as i32, ((height * (rows + 1) as i64) - 10) as i32);
			} else {
				afficher_erreur_fatale("resize-window", "Erreur : width ou height est <= 0");
				std::process::exit(EXIT_FAILURE);
			}

			None
		});

		// Connecter le signal "window-title-changed" avec connect_unsafe
		let _handler_id_title = terminal_widget.connect_unsafe("window-title-changed", false, move |_| {
			let title_ptr = vte_sys::vte_terminal_get_window_title(terminal);
			if !title_ptr.is_null() {
				let title = CStr::from_ptr(title_ptr).to_string_lossy();

				// On utilise le clone ici, ce qui évite de bloquer la variable `window` d'origine
				window_clone.set_title(&title);
			}
			None
		});

		let _close_child = terminal_obj.connect_unsafe("child-exited", false, |_| {
			std::process::exit(EXIT_SUCCESS);
		});

		// let _close_terminal = terminal_obj.connect_unsafe("destroy", false, |_| {
		//	window.close();
		//
		//	None
		// });
	};

	// permet de déplacer et de rester static pour l'application

	window.set_gravity(unsafe { glib::translate::from_glib(GRAVITY_STATIC) });

	window.show_all();

	// Lancer la boucle principale GTK
	gtk::main();
}
