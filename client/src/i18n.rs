use leptos::prelude::*;
use wish_shared::Lang;

const STORAGE_KEY: &str = "wish.lang";

/// Detect language from `?lang=xx` query → localStorage → `navigator.languages`.
pub fn detect_lang() -> Lang {
    if let Some(win) = web_sys::window() {
        // 1. ?lang=xx query override
        if let Ok(search) = win.location().search() {
            let trimmed = search.trim_start_matches('?');
            for part in trimmed.split('&') {
                if let Some(v) = part.strip_prefix("lang=")
                    && let Some(l) = Lang::from_code(v)
                {
                    return l;
                }
            }
        }
        // 2. localStorage
        if let Ok(Some(storage)) = win.local_storage()
            && let Ok(Some(v)) = storage.get_item(STORAGE_KEY)
            && let Some(l) = Lang::from_code(&v)
        {
            return l;
        }
        // 3. navigator.languages
        let nav = win.navigator();
        let langs = nav.languages();
        for i in 0..langs.length() {
            if let Some(tag) = langs.get(i).as_string()
                && let Some(l) = Lang::from_browser_tag(&tag)
            {
                return l;
            }
        }
        if let Ok(tag) = nav.language().ok_or(())
            && let Some(l) = Lang::from_browser_tag(&tag)
        {
            return l;
        }
    }
    Lang::En
}

pub fn save_lang(lang: Lang) {
    if let Some(win) = web_sys::window()
        && let Ok(Some(storage)) = win.local_storage()
    {
        let _ = storage.set_item(STORAGE_KEY, lang.code());
    }
}

/// Leptos context accessor. Panics if not wrapped in a provider — call this
/// only from within `App`.
pub fn use_lang() -> ReadSignal<Lang> {
    use_context::<ReadSignal<Lang>>().expect("Lang context missing — wrap with LangProvider")
}

pub fn use_set_lang() -> WriteSignal<Lang> {
    use_context::<WriteSignal<Lang>>().expect("Lang setter missing — wrap with LangProvider")
}

pub fn translations(lang: Lang) -> &'static Translations {
    match lang {
        Lang::En => &EN,
        Lang::Fr => &FR,
        Lang::It => &IT,
        Lang::De => &DE,
    }
}

/// Every user-facing string in the app. Adding a field requires filling it in
/// for all three languages (compile-time check).
pub struct Translations {
    // ── Common / nav ──────────────────────────────────────────────
    pub nav_home: &'static str,
    pub nav_help: &'static str,
    pub nav_offline: &'static str,
    pub error: &'static str,
    pub saved: &'static str,
    pub failed_to_load: &'static str, // "Failed to load: "

    // ── Home page ─────────────────────────────────────────────────
    pub home_tagline: &'static str,
    pub home_description: &'static str,
    pub home_first_time_prefix: &'static str,
    pub home_first_time_link: &'static str,
    pub home_first_time_suffix: &'static str,
    pub home_offline_prefix: &'static str,
    pub home_offline_link: &'static str,
    pub home_offline_suffix: &'static str,
    pub home_activity_name: &'static str,
    pub home_num_slots: &'static str,
    pub home_slot_name: &'static str,
    pub home_min: &'static str,
    pub home_max: &'static str,
    pub home_slot_placeholder: &'static str, // "Slot {n}"
    pub home_admin_email: &'static str,
    pub home_participant_emails: &'static str,
    pub home_participant_emails_placeholder: &'static str,
    pub home_participant_count_suffix: &'static str, // " participant(s)"
    pub home_customize_emails_note: &'static str,
    pub home_create: &'static str,
    pub home_creating: &'static str,
    pub home_err_activity_required: &'static str,
    pub home_err_admin_required: &'static str,
    pub home_err_participants_required: &'static str,

    // ── Wish page ─────────────────────────────────────────────────
    pub wish_activity: &'static str,
    pub wish_save: &'static str,
    pub wish_saving: &'static str,
    pub wish_loading: &'static str,
    pub wish_saved_body: &'static str,

    // ── Admin page ────────────────────────────────────────────────
    pub admin_problem_settings: &'static str,
    pub admin_email_templates: &'static str,
    pub admin_templates_hint: &'static str, // explaining highlighted $vars
    pub admin_invite_heading: &'static str,
    pub admin_update_heading: &'static str,
    pub admin_reminder_heading: &'static str,
    pub admin_results_heading: &'static str,
    pub admin_available_prefix: &'static str, // "Available: "
    pub admin_save: &'static str,
    pub admin_save_and_send: &'static str,
    pub admin_send_reminder: &'static str,
    pub admin_compute_assignment: &'static str,
    pub admin_assignment: &'static str,
    pub admin_send_results: &'static str,
    pub admin_ws_banner_suffix: &'static str, // "... modified their wish. Reload to see changes."
    pub admin_click_to_reload: &'static str,
    pub admin_parse_errors: &'static str,
    pub admin_fix_errors: &'static str,
    pub admin_err_required_title: &'static str, // "Required variables missing"
    pub admin_saved_and_sending: &'static str,
    pub admin_data_saved: &'static str,
    pub admin_data_saved_sending: &'static str,
    pub admin_reminders_title: &'static str,
    pub admin_reminders_sending: &'static str, // fmt "Sending {n} reminders..."
    pub admin_results_title: &'static str,
    pub admin_results_sending: &'static str, // fmt "Sending {n} result emails..."
    pub admin_no_results: &'static str,
    pub admin_mail_status: &'static str,
    pub admin_error_prefix: &'static str,

    // ── Offline page ──────────────────────────────────────────────
    pub offline_heading: &'static str,
    pub offline_note: &'static str,

    // ── History page ──────────────────────────────────────────────
    pub history_heading: &'static str,
    pub history_password: &'static str,
    pub history_view: &'static str,
    pub history_admin_label: &'static str,  // "admin: "
    pub history_participants: &'static str, // "participants"

    // ── Help page ─────────────────────────────────────────────────
    pub help_heading: &'static str,
    pub help_markdown: &'static str,
}

pub const EN: Translations = Translations {
    nav_home: "Home",
    nav_help: "Help",
    nav_offline: "Offline",
    error: "Error",
    saved: "Saved",
    failed_to_load: "Failed to load: ",

    home_tagline: "Distributes people in various slots maximizing the global satisfaction, taking into account quotas for each slot.",
    home_description: "Organize the groups for various activities according to the desires of your friends, prepare the schedule of an oral exam taking into account the wishes of the students, plan who does what in the organization of a party, ...",
    home_first_time_prefix: "If you are using Wish for the first time, take a look at ",
    home_first_time_link: "the help page",
    home_first_time_suffix: ".",
    home_offline_prefix: "If you don't need to contact the participants by email you can use the ",
    home_offline_link: "offline version",
    home_offline_suffix: ".",
    home_activity_name: "Activity name",
    home_num_slots: "Number of slots",
    home_slot_name: "Slot name",
    home_min: "Min",
    home_max: "Max",
    home_slot_placeholder: "Slot ",
    home_admin_email: "Admin email",
    home_participant_emails: "Participant emails",
    home_participant_emails_placeholder: "first@mail, second@mail, ...",
    home_participant_count_suffix: " participant(s)",
    home_customize_emails_note: "After creating the event you'll be able to customize the invitation, reminder, and result emails on the admin page.",
    home_create: "Create",
    home_creating: "Creating...",
    home_err_activity_required: "Activity name is required",
    home_err_admin_required: "Admin email is required",
    home_err_participants_required: "At least one participant email is required",

    wish_activity: "Activity: ",
    wish_save: "Save",
    wish_saving: "Saving...",
    wish_loading: "Loading...",
    wish_saved_body: "Your wish has been saved.",

    admin_problem_settings: "Problem Settings",
    admin_email_templates: "Email Templates",
    admin_templates_hint: "Highlighted variables are substituted at send time. Unknown $words render literally. Saved when you click Save.",
    admin_invite_heading: "Invite (first email)",
    admin_update_heading: "Update (after slots change)",
    admin_reminder_heading: "Reminder",
    admin_results_heading: "Results (per participant)",
    admin_available_prefix: "Available: ",
    admin_save: "Save",
    admin_save_and_send: "Save & Send Mails",
    admin_send_reminder: "Send Reminder",
    admin_compute_assignment: "Compute Assignment",
    admin_assignment: "Assignment",
    admin_send_results: "Send Results",
    admin_ws_banner_suffix: " modified their wish. Reload to see changes.",
    admin_click_to_reload: " (click to reload)",
    admin_parse_errors: "Parse errors",
    admin_fix_errors: "Fix errors before computing.",
    admin_err_required_title: "Required variables missing",
    admin_saved_and_sending: "Saved & sending",
    admin_data_saved: "Data saved.",
    admin_data_saved_sending: "Data saved. Sending mails...",
    admin_reminders_title: "Reminders",
    admin_reminders_sending: " reminders...",
    admin_results_title: "Results",
    admin_results_sending: " result emails...",
    admin_no_results: "No results to send. Compute assignment first.",
    admin_mail_status: "Mail status",
    admin_error_prefix: "Error: ",

    offline_heading: "Wish \u{2014} Offline",
    offline_note: "This is the offline version. No emails are sent and no data is saved on the server.",

    history_heading: "Wish \u{2014} History",
    history_password: "Password",
    history_view: "View History",
    history_admin_label: "admin: ",
    history_participants: "participants",

    help_heading: "Help",
    help_markdown: HELP_MD_EN,
};

pub const FR: Translations = Translations {
    nav_home: "Accueil",
    nav_help: "Aide",
    nav_offline: "Hors ligne",
    error: "Erreur",
    saved: "Enregistré",
    failed_to_load: "Échec du chargement : ",

    home_tagline: "Répartit les personnes dans différents créneaux en maximisant la satisfaction globale, en tenant compte des quotas de chaque créneau.",
    home_description: "Organisez les groupes pour diverses activités selon les envies de vos amis, préparez le planning d'un examen oral en tenant compte des souhaits des étudiants, planifiez qui fait quoi dans l'organisation d'une fête, …",
    home_first_time_prefix: "Si vous utilisez Wish pour la première fois, consultez ",
    home_first_time_link: "la page d'aide",
    home_first_time_suffix: ".",
    home_offline_prefix: "Si vous n'avez pas besoin de contacter les participants par e-mail, vous pouvez utiliser la ",
    home_offline_link: "version hors ligne",
    home_offline_suffix: ".",
    home_activity_name: "Nom de l'activité",
    home_num_slots: "Nombre de créneaux",
    home_slot_name: "Nom du créneau",
    home_min: "Min",
    home_max: "Max",
    home_slot_placeholder: "Créneau ",
    home_admin_email: "E-mail de l'administrateur",
    home_participant_emails: "E-mails des participants",
    home_participant_emails_placeholder: "premier@mail, deuxieme@mail, …",
    home_participant_count_suffix: " participant(s)",
    home_customize_emails_note: "Après avoir créé l'événement, vous pourrez personnaliser les e-mails d'invitation, de rappel et de résultat sur la page d'administration.",
    home_create: "Créer",
    home_creating: "Création…",
    home_err_activity_required: "Le nom de l'activité est requis",
    home_err_admin_required: "L'e-mail de l'administrateur est requis",
    home_err_participants_required: "Au moins un e-mail de participant est requis",

    wish_activity: "Activité : ",
    wish_save: "Enregistrer",
    wish_saving: "Enregistrement…",
    wish_loading: "Chargement…",
    wish_saved_body: "Votre souhait a été enregistré.",

    admin_problem_settings: "Paramètres du problème",
    admin_email_templates: "Modèles d'e-mail",
    admin_templates_hint: "Les variables surlignées sont remplacées à l'envoi. Les $mots inconnus sont rendus tels quels. Enregistrés lorsque vous cliquez sur Enregistrer.",
    admin_invite_heading: "Invitation (premier e-mail)",
    admin_update_heading: "Mise à jour (après modification des créneaux)",
    admin_reminder_heading: "Rappel",
    admin_results_heading: "Résultats (par participant)",
    admin_available_prefix: "Disponible : ",
    admin_save: "Enregistrer",
    admin_save_and_send: "Enregistrer et envoyer",
    admin_send_reminder: "Envoyer un rappel",
    admin_compute_assignment: "Calculer l'affectation",
    admin_assignment: "Affectation",
    admin_send_results: "Envoyer les résultats",
    admin_ws_banner_suffix: " a modifié son souhait. Rechargez pour voir les changements.",
    admin_click_to_reload: " (cliquer pour recharger)",
    admin_parse_errors: "Erreurs de syntaxe",
    admin_fix_errors: "Corrigez les erreurs avant de calculer.",
    admin_err_required_title: "Variables requises manquantes",
    admin_saved_and_sending: "Enregistré et envoi en cours",
    admin_data_saved: "Données enregistrées.",
    admin_data_saved_sending: "Données enregistrées. Envoi des e-mails…",
    admin_reminders_title: "Rappels",
    admin_reminders_sending: " rappels…",
    admin_results_title: "Résultats",
    admin_results_sending: " e-mails de résultat…",
    admin_no_results: "Aucun résultat à envoyer. Calculez d'abord l'affectation.",
    admin_mail_status: "Statut des e-mails",
    admin_error_prefix: "Erreur : ",

    offline_heading: "Wish \u{2014} Hors ligne",
    offline_note: "Ceci est la version hors ligne. Aucun e-mail n'est envoyé et aucune donnée n'est enregistrée sur le serveur.",

    history_heading: "Wish \u{2014} Historique",
    history_password: "Mot de passe",
    history_view: "Voir l'historique",
    history_admin_label: "admin : ",
    history_participants: "participants",

    help_heading: "Aide",
    help_markdown: HELP_MD_FR,
};

pub const IT: Translations = Translations {
    nav_home: "Home",
    nav_help: "Aiuto",
    nav_offline: "Offline",
    error: "Errore",
    saved: "Salvato",
    failed_to_load: "Caricamento non riuscito: ",

    home_tagline: "Distribuisce le persone in diverse fasce massimizzando la soddisfazione complessiva, tenendo conto dei limiti di ciascuna fascia.",
    home_description: "Organizza i gruppi per varie attività in base ai desideri dei tuoi amici, prepara il calendario di un esame orale tenendo conto delle preferenze degli studenti, pianifica chi fa cosa nell'organizzazione di una festa, …",
    home_first_time_prefix: "Se è la prima volta che usi Wish, dai un'occhiata alla ",
    home_first_time_link: "pagina di aiuto",
    home_first_time_suffix: ".",
    home_offline_prefix: "Se non hai bisogno di contattare i partecipanti via e-mail puoi usare la ",
    home_offline_link: "versione offline",
    home_offline_suffix: ".",
    home_activity_name: "Nome dell'attività",
    home_num_slots: "Numero di fasce",
    home_slot_name: "Nome della fascia",
    home_min: "Min",
    home_max: "Max",
    home_slot_placeholder: "Fascia ",
    home_admin_email: "E-mail dell'amministratore",
    home_participant_emails: "E-mail dei partecipanti",
    home_participant_emails_placeholder: "primo@mail, secondo@mail, …",
    home_participant_count_suffix: " partecipante/i",
    home_customize_emails_note: "Dopo aver creato l'evento potrai personalizzare le e-mail di invito, promemoria e risultato dalla pagina di amministrazione.",
    home_create: "Crea",
    home_creating: "Creazione…",
    home_err_activity_required: "Il nome dell'attività è obbligatorio",
    home_err_admin_required: "L'e-mail dell'amministratore è obbligatoria",
    home_err_participants_required: "È richiesta almeno un'e-mail di un partecipante",

    wish_activity: "Attività: ",
    wish_save: "Salva",
    wish_saving: "Salvataggio…",
    wish_loading: "Caricamento…",
    wish_saved_body: "La tua preferenza è stata salvata.",

    admin_problem_settings: "Impostazioni del problema",
    admin_email_templates: "Modelli e-mail",
    admin_templates_hint: "Le variabili evidenziate vengono sostituite all'invio. Le $parole sconosciute vengono scritte così come sono. Salvate quando fai clic su Salva.",
    admin_invite_heading: "Invito (prima e-mail)",
    admin_update_heading: "Aggiornamento (dopo modifica fasce)",
    admin_reminder_heading: "Promemoria",
    admin_results_heading: "Risultati (per partecipante)",
    admin_available_prefix: "Disponibili: ",
    admin_save: "Salva",
    admin_save_and_send: "Salva e invia",
    admin_send_reminder: "Invia promemoria",
    admin_compute_assignment: "Calcola assegnazione",
    admin_assignment: "Assegnazione",
    admin_send_results: "Invia risultati",
    admin_ws_banner_suffix: " ha modificato la sua preferenza. Ricarica per vedere i cambiamenti.",
    admin_click_to_reload: " (clicca per ricaricare)",
    admin_parse_errors: "Errori di sintassi",
    admin_fix_errors: "Correggi gli errori prima di calcolare.",
    admin_err_required_title: "Variabili obbligatorie mancanti",
    admin_saved_and_sending: "Salvato e invio in corso",
    admin_data_saved: "Dati salvati.",
    admin_data_saved_sending: "Dati salvati. Invio e-mail…",
    admin_reminders_title: "Promemoria",
    admin_reminders_sending: " promemoria…",
    admin_results_title: "Risultati",
    admin_results_sending: " e-mail di risultato…",
    admin_no_results: "Nessun risultato da inviare. Calcola prima l'assegnazione.",
    admin_mail_status: "Stato e-mail",
    admin_error_prefix: "Errore: ",

    offline_heading: "Wish \u{2014} Offline",
    offline_note: "Questa è la versione offline. Non vengono inviate e-mail e nessun dato viene salvato sul server.",

    history_heading: "Wish \u{2014} Cronologia",
    history_password: "Password",
    history_view: "Mostra cronologia",
    history_admin_label: "admin: ",
    history_participants: "partecipanti",

    help_heading: "Aiuto",
    help_markdown: HELP_MD_IT,
};

pub const DE: Translations = Translations {
    nav_home: "Start",
    nav_help: "Hilfe",
    nav_offline: "Offline",
    error: "Fehler",
    saved: "Gespeichert",
    failed_to_load: "Laden fehlgeschlagen: ",

    home_tagline: "Verteilt Personen auf verschiedene Zeitfenster und maximiert die Gesamtzufriedenheit unter Berücksichtigung der Kontingente jedes Zeitfensters.",
    home_description: "Organisiere die Gruppen für verschiedene Aktivitäten nach den Wünschen deiner Freunde, plane den Zeitplan einer mündlichen Prüfung unter Berücksichtigung der Wünsche der Studierenden, plane, wer was bei der Organisation einer Feier übernimmt, …",
    home_first_time_prefix: "Wenn du Wish zum ersten Mal verwendest, sieh dir ",
    home_first_time_link: "die Hilfeseite",
    home_first_time_suffix: " an.",
    home_offline_prefix: "Wenn du die Teilnehmer nicht per E-Mail kontaktieren musst, kannst du die ",
    home_offline_link: "Offline-Version",
    home_offline_suffix: " verwenden.",
    home_activity_name: "Name der Aktivität",
    home_num_slots: "Anzahl der Zeitfenster",
    home_slot_name: "Name des Zeitfensters",
    home_min: "Min",
    home_max: "Max",
    home_slot_placeholder: "Zeitfenster ",
    home_admin_email: "Admin-E-Mail",
    home_participant_emails: "E-Mails der Teilnehmer",
    home_participant_emails_placeholder: "erste@mail, zweite@mail, …",
    home_participant_count_suffix: " Teilnehmer",
    home_customize_emails_note: "Nach dem Erstellen der Veranstaltung kannst du die Einladungs-, Erinnerungs- und Ergebnis-E-Mails auf der Admin-Seite anpassen.",
    home_create: "Erstellen",
    home_creating: "Wird erstellt…",
    home_err_activity_required: "Name der Aktivität ist erforderlich",
    home_err_admin_required: "Admin-E-Mail ist erforderlich",
    home_err_participants_required: "Mindestens eine Teilnehmer-E-Mail ist erforderlich",

    wish_activity: "Aktivität: ",
    wish_save: "Speichern",
    wish_saving: "Speichern…",
    wish_loading: "Laden…",
    wish_saved_body: "Dein Wunsch wurde gespeichert.",

    admin_problem_settings: "Problemeinstellungen",
    admin_email_templates: "E-Mail-Vorlagen",
    admin_templates_hint: "Hervorgehobene Variablen werden beim Versand ersetzt. Unbekannte $Wörter werden wörtlich ausgegeben. Gespeichert, wenn du auf Speichern klickst.",
    admin_invite_heading: "Einladung (erste E-Mail)",
    admin_update_heading: "Aktualisierung (nach Änderung der Zeitfenster)",
    admin_reminder_heading: "Erinnerung",
    admin_results_heading: "Ergebnisse (pro Teilnehmer)",
    admin_available_prefix: "Verfügbar: ",
    admin_save: "Speichern",
    admin_save_and_send: "Speichern und senden",
    admin_send_reminder: "Erinnerung senden",
    admin_compute_assignment: "Zuteilung berechnen",
    admin_assignment: "Zuteilung",
    admin_send_results: "Ergebnisse senden",
    admin_ws_banner_suffix: " hat seinen Wunsch geändert. Lade neu, um die Änderungen zu sehen.",
    admin_click_to_reload: " (klicken, um neu zu laden)",
    admin_parse_errors: "Syntaxfehler",
    admin_fix_errors: "Fehler vor dem Berechnen beheben.",
    admin_err_required_title: "Erforderliche Variablen fehlen",
    admin_saved_and_sending: "Gespeichert, wird gesendet",
    admin_data_saved: "Daten gespeichert.",
    admin_data_saved_sending: "Daten gespeichert. E-Mails werden gesendet…",
    admin_reminders_title: "Erinnerungen",
    admin_reminders_sending: " Erinnerungen…",
    admin_results_title: "Ergebnisse",
    admin_results_sending: " Ergebnis-E-Mails…",
    admin_no_results: "Keine Ergebnisse zum Senden. Berechne zuerst die Zuteilung.",
    admin_mail_status: "E-Mail-Status",
    admin_error_prefix: "Fehler: ",

    offline_heading: "Wish \u{2014} Offline",
    offline_note: "Dies ist die Offline-Version. Es werden keine E-Mails gesendet und keine Daten auf dem Server gespeichert.",

    history_heading: "Wish \u{2014} Verlauf",
    history_password: "Passwort",
    history_view: "Verlauf anzeigen",
    history_admin_label: "Admin: ",
    history_participants: "Teilnehmer",

    help_heading: "Hilfe",
    help_markdown: HELP_MD_DE,
};

// ── Help markdown (per language) ──────────────────────────────────
//
// Supported subset (rendered by `crate::components::markdown::render`):
//   # / ## / ### / ####  headings
//   paragraphs separated by blank lines
//   - / * bullet lists
//   **bold** *italic*
//   [text](url) links
// HTML characters are escaped except for the emitted tags.

const HELP_MD_EN: &str = "\
## What does Wish do?

### Aim of the algorithm

The algorithm takes as input a matrix whose lines are the users and whose columns are the slots.

The matrix is filled in with non-negative integers describing the wishes of the users (\"grades\"). \
A small number means the user would be very satisfied if put in this slot, and a high number means \
they would be disappointed. For a given arrangement, the \"penalty\" is the sum of the squares of \
each user's grade for their assigned slot.

A maximum and minimum number of users is associated to each slot. We use the \
[Hungarian algorithm](https://en.wikipedia.org/wiki/Hungarian_algorithm) to minimize the penalty \
under these constraints.

### Ensuring fair choices

To prevent gaming the system (e.g., rating all slots as \"hated\" except one), fairness rules are enforced. With n slots:

- Up to 1 slot can have grade n-1 (hated)
- Up to 2 slots can have grade ≥ n-2
- Up to 3 slots can have grade ≥ n-3
- ...
- Up to n slots can have grade ≥ 0

The default setting is grade 0 for all slots, meaning no preference.

When setting wishes, the sliders automatically prevent unfair combinations.

### Users with special constraints

If someone must not be put in certain slots (e.g., another exam at that time), the administrator \
can edit their wishes directly on the admin page, setting a very high grade for the avoided slots.

### Private URLs

Each participant gets a unique private URL to prevent peeking at others' wishes.

## How to use

### Create an event

Fill in the activity name, define slots (with min/max participant counts), enter the admin email \
and participant emails, then click Create.

The admin receives an email with a link to the admin page.

### Admin page

The main area is a text editor showing slots and participants in a structured format. You can edit \
slot names, quotas, add/remove participants, and modify wishes.

**Save** — saves changes to the server.

**Save & Send Mails** — saves and sends invitation emails to new participants (or update emails \
if slots changed).

**Send Reminder** — sends a reminder to participants who haven't filled their wishes.

**Compute Assignment** — runs the Hungarian algorithm and shows results.

**Send Results** — emails each participant their assigned slot.

### Offline mode

If you don't need email functionality, use the [offline version](/offline). Enter all data \
manually and compute assignments locally.

## What Wish doesn't do

**Wish isn't Doodle** — the aim is to distribute people across slots, not put everyone in the same one.

**One slot per person** — each participant is assigned to exactly one slot.
";

const HELP_MD_FR: &str = "\
## Que fait Wish ?

### Objectif de l'algorithme

L'algorithme prend en entrée une matrice dont les lignes sont les utilisateurs et les colonnes les créneaux.

La matrice contient des entiers positifs ou nuls décrivant les souhaits des utilisateurs (« notes »). \
Une petite valeur signifie que l'utilisateur serait très satisfait d'être placé dans ce créneau, une \
grande valeur qu'il serait déçu. Pour une affectation donnée, la « pénalité » est la somme des carrés \
des notes de chaque utilisateur pour son créneau attribué.

Un nombre maximum et minimum d'utilisateurs est associé à chaque créneau. Nous utilisons l'\
[algorithme hongrois](https://en.wikipedia.org/wiki/Hungarian_algorithm) pour minimiser la pénalité \
sous ces contraintes.

### Garantir des choix équitables

Pour éviter de fausser le système (par exemple en notant tous les créneaux « détestés » sauf un), \
des règles d'équité sont appliquées. Avec n créneaux :

- Au plus 1 créneau peut avoir la note n-1 (détesté)
- Au plus 2 créneaux peuvent avoir une note ≥ n-2
- Au plus 3 créneaux peuvent avoir une note ≥ n-3
- ...
- Au plus n créneaux peuvent avoir une note ≥ 0

La valeur par défaut est 0 pour tous les créneaux, c'est-à-dire aucune préférence.

Lors de la saisie des souhaits, les curseurs empêchent automatiquement les combinaisons non équitables.

### Utilisateurs avec contraintes particulières

Si une personne ne doit pas être placée dans certains créneaux (par exemple un autre examen en même \
temps), l'administrateur peut modifier ses souhaits directement sur la page d'administration en \
attribuant une note très élevée aux créneaux à éviter.

### URLs privées

Chaque participant reçoit une URL privée unique pour éviter de voir les souhaits des autres.

## Comment l'utiliser

### Créer un événement

Renseignez le nom de l'activité, définissez les créneaux (avec leurs quotas min/max), saisissez \
l'e-mail de l'administrateur et ceux des participants, puis cliquez sur Créer.

L'administrateur reçoit un e-mail contenant un lien vers la page d'administration.

### Page d'administration

La zone principale est un éditeur de texte affichant les créneaux et les participants dans un format \
structuré. Vous pouvez modifier les noms de créneaux, les quotas, ajouter/supprimer des participants \
et modifier les souhaits.

**Enregistrer** — sauvegarde les changements sur le serveur.

**Enregistrer et envoyer** — sauvegarde et envoie les e-mails d'invitation aux nouveaux participants \
(ou des e-mails de mise à jour si les créneaux ont changé).

**Envoyer un rappel** — envoie un rappel aux participants qui n'ont pas encore rempli leurs souhaits.

**Calculer l'affectation** — exécute l'algorithme hongrois et affiche les résultats.

**Envoyer les résultats** — envoie à chaque participant le créneau qui lui est attribué.

### Mode hors ligne

Si vous n'avez pas besoin de la fonctionnalité e-mail, utilisez la [version hors ligne](/offline). \
Saisissez toutes les données manuellement et calculez l'affectation localement.

## Ce que Wish ne fait pas

**Wish n'est pas Doodle** — le but est de répartir les personnes entre les créneaux, pas de les \
regrouper toutes dans un seul.

**Un seul créneau par personne** — chaque participant est affecté à exactement un créneau.
";

const HELP_MD_IT: &str = "\
## Cosa fa Wish?

### Scopo dell'algoritmo

L'algoritmo prende in ingresso una matrice le cui righe sono gli utenti e le cui colonne sono le fasce.

La matrice è riempita con numeri interi non negativi che descrivono le preferenze degli utenti \
(«voti»). Un numero basso significa che l'utente sarebbe molto soddisfatto se assegnato a quella \
fascia, un numero alto che ne sarebbe deluso. Per una data assegnazione, la «penalità» è la somma \
dei quadrati del voto di ogni utente per la fascia che gli è stata assegnata.

A ogni fascia è associato un numero minimo e massimo di utenti. Utilizziamo l'\
[algoritmo ungherese](https://en.wikipedia.org/wiki/Hungarian_algorithm) per minimizzare la penalità \
rispettando questi vincoli.

### Garantire scelte eque

Per evitare di manipolare il sistema (ad esempio valutando tutte le fasce come «odiate» tranne una), \
vengono applicate regole di equità. Con n fasce:

- Al massimo 1 fascia può avere voto n-1 (odiata)
- Al massimo 2 fasce possono avere voto ≥ n-2
- Al massimo 3 fasce possono avere voto ≥ n-3
- ...
- Al massimo n fasce possono avere voto ≥ 0

Il valore predefinito è 0 per tutte le fasce, ossia nessuna preferenza.

Quando si impostano le preferenze, i cursori impediscono automaticamente le combinazioni non eque.

### Utenti con vincoli particolari

Se una persona non deve essere inserita in certe fasce (ad esempio un altro esame in quell'orario), \
l'amministratore può modificare le sue preferenze direttamente dalla pagina di amministrazione, \
impostando un voto molto alto per le fasce da evitare.

### URL private

Ogni partecipante riceve una URL privata e univoca per evitare che veda le preferenze altrui.

## Come usarlo

### Creare un evento

Compila il nome dell'attività, definisci le fasce (con i limiti min/max di partecipanti), inserisci \
l'e-mail dell'amministratore e quelle dei partecipanti, poi fai clic su Crea.

L'amministratore riceve un'e-mail con il link alla pagina di amministrazione.

### Pagina di amministrazione

L'area principale è un editor di testo che mostra fasce e partecipanti in un formato strutturato. \
Puoi modificare i nomi delle fasce, i limiti, aggiungere/rimuovere partecipanti e modificare le preferenze.

**Salva** — salva le modifiche sul server.

**Salva e invia** — salva e invia le e-mail di invito ai nuovi partecipanti (o e-mail di aggiornamento \
se le fasce sono cambiate).

**Invia promemoria** — invia un promemoria ai partecipanti che non hanno ancora indicato le preferenze.

**Calcola assegnazione** — esegue l'algoritmo ungherese e mostra i risultati.

**Invia risultati** — invia a ogni partecipante la fascia assegnata.

### Modalità offline

Se non hai bisogno della funzione e-mail, usa la [versione offline](/offline). Inserisci tutti i \
dati manualmente e calcola l'assegnazione localmente.

## Cosa Wish non fa

**Wish non è Doodle** — lo scopo è distribuire le persone tra le fasce, non metterle tutte nella stessa.

**Una fascia per persona** — ogni partecipante viene assegnato esattamente a una fascia.
";

const HELP_MD_DE: &str = "\
## Was macht Wish?

### Ziel des Algorithmus

Der Algorithmus nimmt als Eingabe eine Matrix, deren Zeilen die Benutzer und deren Spalten die Zeitfenster sind.

Die Matrix enthält nicht-negative ganze Zahlen, die die Wünsche der Benutzer beschreiben (\"Noten\"). \
Eine kleine Zahl bedeutet, dass der Benutzer sehr zufrieden wäre, diesem Zeitfenster zugeteilt zu \
werden; eine hohe Zahl bedeutet, dass er enttäuscht wäre. Für eine gegebene Zuteilung ist der \
\"Nachteil\" die Summe der Quadrate der Noten jedes Benutzers für sein zugeteiltes Zeitfenster.

Jedem Zeitfenster ist eine minimale und maximale Anzahl von Benutzern zugeordnet. Wir verwenden \
den [ungarischen Algorithmus](https://en.wikipedia.org/wiki/Hungarian_algorithm), um den Nachteil \
unter diesen Bedingungen zu minimieren.

### Faire Auswahl sicherstellen

Um zu verhindern, dass das System ausgetrickst wird (z. B. indem alle Zeitfenster außer einem als \
\"gehasst\" bewertet werden), werden Fairnessregeln durchgesetzt. Bei n Zeitfenstern:

- Höchstens 1 Zeitfenster darf die Note n-1 haben (gehasst)
- Höchstens 2 Zeitfenster dürfen eine Note ≥ n-2 haben
- Höchstens 3 Zeitfenster dürfen eine Note ≥ n-3 haben
- ...
- Höchstens n Zeitfenster dürfen eine Note ≥ 0 haben

Die Standardeinstellung ist Note 0 für alle Zeitfenster, d. h. keine Präferenz.

Beim Setzen der Wünsche verhindern die Schieberegler automatisch unfaire Kombinationen.

### Benutzer mit besonderen Einschränkungen

Wenn eine Person bestimmten Zeitfenstern nicht zugeteilt werden darf (z. B. eine andere Prüfung zur \
gleichen Zeit), kann der Administrator ihre Wünsche direkt auf der Admin-Seite bearbeiten und für \
die zu vermeidenden Zeitfenster eine sehr hohe Note vergeben.

### Private URLs

Jeder Teilnehmer erhält eine eindeutige private URL, damit niemand die Wünsche der anderen einsehen kann.

## Wie man es verwendet

### Eine Veranstaltung erstellen

Trage den Namen der Aktivität ein, definiere die Zeitfenster (mit min./max. Teilnehmerzahl), gib die \
Admin-E-Mail und die E-Mails der Teilnehmer ein und klicke auf Erstellen.

Der Administrator erhält eine E-Mail mit einem Link zur Admin-Seite.

### Admin-Seite

Der Hauptbereich ist ein Texteditor, der Zeitfenster und Teilnehmer in einem strukturierten Format \
anzeigt. Du kannst Namen der Zeitfenster, Kontingente, Teilnehmer (hinzufügen/entfernen) und Wünsche \
bearbeiten.

**Speichern** — speichert die Änderungen auf dem Server.

**Speichern und senden** — speichert und sendet Einladungs-E-Mails an neue Teilnehmer (oder \
Aktualisierungs-E-Mails, wenn sich die Zeitfenster geändert haben).

**Erinnerung senden** — sendet eine Erinnerung an Teilnehmer, die ihre Wünsche noch nicht eingetragen haben.

**Zuteilung berechnen** — führt den ungarischen Algorithmus aus und zeigt die Ergebnisse an.

**Ergebnisse senden** — sendet jedem Teilnehmer sein zugeteiltes Zeitfenster per E-Mail.

### Offline-Modus

Wenn du die E-Mail-Funktion nicht benötigst, verwende die [Offline-Version](/offline). Gib alle \
Daten manuell ein und berechne die Zuteilung lokal.

## Was Wish nicht macht

**Wish ist nicht Doodle** — das Ziel ist, Personen auf Zeitfenster zu verteilen, nicht alle in dasselbe zu setzen.

**Ein Zeitfenster pro Person** — jeder Teilnehmer wird genau einem Zeitfenster zugeteilt.
";
