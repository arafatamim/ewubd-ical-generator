const root = document.getElementById("root");

const calendarRemotePath = new URL(document.location).searchParams.get(
  "calendar_path"
);

const fetchCalendar = () =>
  fetch("/api/entries?calendar_path=" + calendarRemotePath, {
    credentials: "include",
    mode: "no-cors",
  });

const calendar = await fetchCalendar()
  .then((res) => {
    if (!res.ok) {
      root.innerHTML = `<p>Couldn't generate calendar :-( <br />Check if <a href="//ewubd.edu">ewubd.edu</a> loads correctly</p>`;
    } else return res.json();
  })
  .catch((e) => {
    root.innerHTML = `<p>Couldn't generate calendar :-( <br />(${e.message})</p>`;
  });

if (calendar != null) {
  const calendarName = calendar["calendar_name"];
  const revisedDate = new Date(calendar["revised_date"]);
  const semester = calendar["semester"];
  const year = calendar["year"];
  const fileUri = (protocol) =>
    protocol +
    window.location.host +
    "/api/generate?calendar_path=" +
    calendarRemotePath;

  document.title = `${semester} ${year} calendar - EWU Calendar`;
  const doc = `<h2>${semester} ${year}</h2>
        <h3>${calendarName}</h3>
        <p>Last updated: ${revisedDate.toDateString()}</p>
        <p>
          <ul>
            <li><a class="button" target="_blank" href="https://calendar.google.com/calendar/u/0/r?cid=${fileUri(
              "webcal://"
            )}">Add to Google Calendar</a></li>
            <li><a class="button" target="_blank" href="${fileUri(
              "webcal://"
            )}">Add to Apple Calendar</a></li>
            <li><a class="button" target="_blank" href="https://outlook.live.com/calendar/0/addfromweb/?url=${fileUri(
              "https://"
            )}">Add to Outlook.com</a></li>
            <li>or <a href="/api/generate?calendar_path=${encodeURIComponent(
              calendarRemotePath
            )}">download .ics file</a></li>
          </ul>
        </p>`;

  root.innerHTML = doc;

  const calendarEl = document.getElementById("calendar");

  calendarEl.innerText = "Loading calendar...";

  // load calendar after entries are fetched
  const [
    { Calendar: FullCalendar },
    { default: dayGridPlugin },
    { default: listPlugin },
    { default: tippy },
  ] = await Promise.all([
    import("@fullcalendar/core"),
    import("@fullcalendar/daygrid"),
    import("@fullcalendar/list"),
    import("tippy.js"),
  ]);

  const lastEvent = calendar.entries[calendar.entries.length - 1];
  const lastEventDate = new Date(lastEvent.date[1] ?? lastEvent.date[0]);

  const fullCalendar = new FullCalendar(calendarEl, {
    initialView: "dayGridMonth",
    timeZone: "+6",
    plugins: [dayGridPlugin, listPlugin],
    validRange: {
      start: calendar.entries[0].date[0],
      end: lastEventDate,
    },
    headerToolbar: {
      start: "title",
      end: "dayGridMonth,listYear",
    },
    footerToolbar: {
      right: "prev,next",
    },
    height: "auto",
    displayEventTime: false,
    eventDidMount: (info) => {
      tippy(info.el, {
        trigger: info.view.type !== "listYear" ? "mouseenter" : "manual",
        content: info.event.title,
        placement: "top",
        arrow: false,
        theme: "myTheme",
        delay: [0, 0],
        duration: 0,
      });
      // display event dot only if title contains exam or last day
      if (info.view.type === "listYear") {
        const eventDot = info.el.getElementsByClassName("fc-list-event-dot")[0];
        if (info.event.title.match(/exam|last day/i)) {
          eventDot.style.display = "inline-block";
        } else {
          eventDot.style.display = "none";
        }
      }
    },
    events: calendar.entries.map((entry) => {
      let endDate = new Date(entry.date[1]);
      endDate.setDate(endDate.getDate() + 1); // to make the end date inclusive
      return {
        title: entry.event,
        start: entry.date[0],
        end: endDate,
        allDay: true,
      };
    }),
    eventColor: "var(--pico-primary)",
    eventTextColor: "#000",
  });

  fullCalendar.gotoDate(calendar.entries[0].date[0]); // navigate to the first event

  calendarEl.innerText = "";
  fullCalendar.render();
}
