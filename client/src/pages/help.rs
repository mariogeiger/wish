use leptos::prelude::*;

#[component]
pub fn HelpPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"Help"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/offline">"Offline"</a>
            </nav>

            <h2>"What does Wish do?"</h2>

            <h3>"Aim of the algorithm"</h3>
            <p>"The algorithm takes as input a matrix whose lines are the users and whose columns are the slots."</p>
            <p>"The matrix is filled in with non-negative integers describing the wishes of the users (\"grades\"). \
                A small number means the user would be very satisfied if put in this slot, and a high number means \
                they would be disappointed. For a given arrangement, the \"penalty\" is the sum of the squares of \
                each user's grade for their assigned slot."</p>
            <p>"A maximum and minimum number of users is associated to each slot. We use the "
                <a href="https://en.wikipedia.org/wiki/Hungarian_algorithm">"Hungarian algorithm"</a>
                " to minimize the penalty under these constraints."</p>

            <h3>"Ensuring fair choices"</h3>
            <p>"To prevent gaming the system (e.g., rating all slots as \"hated\" except one), \
                fairness rules are enforced. With n slots:"</p>
            <ul>
                <li>"Up to 1 slot can have grade n-1 (hated)"</li>
                <li>"Up to 2 slots can have grade \u{2265} n-2"</li>
                <li>"Up to 3 slots can have grade \u{2265} n-3"</li>
                <li>"..."</li>
                <li>"Up to n slots can have grade \u{2265} 0"</li>
            </ul>
            <p>"The default setting is grade 0 for all slots, meaning no preference."</p>
            <p>"When setting wishes, the sliders automatically prevent unfair combinations."</p>

            <h3>"Users with special constraints"</h3>
            <p>"If someone must not be put in certain slots (e.g., another exam at that time), \
                the administrator can edit their wishes directly on the admin page, setting a very \
                high grade for the avoided slots."</p>

            <h3>"Private URLs"</h3>
            <p>"Each participant gets a unique private URL to prevent peeking at others' wishes."</p>

            <h2>"How to use"</h2>

            <h3>"Create an event"</h3>
            <p>"Fill in the activity name, define slots (with min/max participant counts), \
                enter the admin email and participant emails, then click Create."</p>
            <p>"The admin receives an email with a link to the admin page."</p>

            <h3>"Admin page"</h3>
            <p>"The main area is a text editor showing slots and participants in a structured format. \
                You can edit slot names, quotas, add/remove participants, and modify wishes."</p>
            <p><strong>"Save"</strong>" — saves changes to the server."</p>
            <p><strong>"Save & Send Mails"</strong>" — saves and sends invitation emails to new participants \
                (or update emails if slots changed)."</p>
            <p><strong>"Send Reminder"</strong>" — sends a reminder to participants who haven't filled their wishes."</p>
            <p><strong>"Compute Assignment"</strong>" — runs the Hungarian algorithm and shows results."</p>
            <p><strong>"Send Results"</strong>" — emails each participant their assigned slot."</p>

            <h3>"Offline mode"</h3>
            <p>"If you don't need email functionality, use the "
                <a href="/offline">"offline version"</a>
                ". Enter all data manually and compute assignments locally."</p>

            <h2>"What Wish doesn't do"</h2>
            <p><strong>"Wish isn't Doodle"</strong>" — the aim is to distribute people across slots, not put everyone in the same one."</p>
            <p><strong>"One slot per person"</strong>" — each participant is assigned to exactly one slot."</p>
        </div>
    }
}
