use crate::core::prelude::*;
use maud::{html, Markup, DOCTYPE};
use rocket::request::FlashMessage;
use std::collections::HashMap;

const LEAFLET_CSS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.css";
const LEAFLET_CSS_SHA512: &str="sha512-puBpdR0798OZvTTbP4A8Ix/l+A4dHDD0DGqYW6RQ+9jxkRFclaxxQb/SJAWZfWAkuyeQUytO7+7N4QKrDh+drA==";
const LEAFLET_JS_URL: &str = "https://cdnjs.cloudflare.com/ajax/libs/leaflet/1.4.0/leaflet.js";
const LEAFLET_JS_SHA512 : &str="sha512-QVftwZFqvtRNi0ZyCtsznlKSWOStnDORoefr1enyq5mVL4tmKB3S/EnC3rRJcxCPavG10IcrVGSmPh6Qw5lwrg==";
const MAIN_CSS_URL: &str = "/main.css";
const MAP_JS_URL: &str = "/map.js";

pub fn index(email: Option<&str>) -> Markup {
    page(
        "OpenFairDB Search",
        None,
        html! {
            (header(email))
            div class="search" {
                h1 {"OpenFairDB Search"}
                (global_search_form(None))
            }
        },
    )
}

fn header(email: Option<&str>) -> Markup {
    html! {
    header {
        @if let Some(email) = email {
            div class="msg" { "Your are logged in as " span class="email" { (email) } }
            nav {
                a href="/" { "search" }
                a href="dashboard" { "dashboard" }
                form class="logout" action="logout" method ="POST" {
                    input type="submit" value="logout";
                }
            }
        }
        @ else {
            nav {
                a href="login"  { "login" }
                a href="register" { "register" }
            }
        }
    }
    }
}

pub struct DashBoardPresenter<'a> {
    pub email: &'a str,
    pub entry_count: usize,
    pub event_count: usize,
    pub tag_count: usize,
    pub user_count: usize,
}

pub fn dashboard(data: DashBoardPresenter) -> Markup {
    page(
        "Admin Dashboard",
        None,
        html! {
            (header(Some(data.email)))
            main {
                h3 { "Database Statistics" }
                table {
                    tr {
                        td {"Number of Entries"}
                        td {(data.entry_count)}
                    }
                    tr {
                        td {"Number of Events"}
                        td {(data.event_count)}
                    }
                    tr {
                        td {"Number of Users"}
                        td {(data.user_count)}
                    }
                    tr {
                        td {"Number of Tags"}
                        td {(data.tag_count)}
                    }
                }
            }
        },
    )
}

pub fn global_search_form(search_term: Option<&str>) -> Markup {
    html! {
        div class="search-form" {
            form action="search" method="GET" {
                input
                    type="text"
                    name="q"
                    value=(search_term.unwrap_or(""))
                    size=(50)
                    maxlength=(200)
                    placeholer="search term, empty = all";
                br;
                input class="btn" type="submit" value="search";
            }
        }
    }
}

fn leaflet_css_link() -> Markup {
    html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="anonymous";
    }
}

pub fn search_results(search_term: &str, entries: &[IndexedEntry]) -> Markup {
    page(
        "OpenFairDB Search Results",
        None,
        html! {
            div class="search" {
                h1 {"OpenFairDB Search"}
                (global_search_form(Some(search_term)))
            }
            div class="results" {
                @if entries.is_empty(){
                    p{ "We are so sorry but we could not find any entries
                        that are related to your search term "
                            em {
                                (format!("'{}'", search_term))
                            }
                    }
                } @else {
                    p {
                        ul class="result-list" {
                            @for e in entries {
                                li{
                                    (entry_result(e))
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

fn entry_result(e: &IndexedEntry) -> Markup {
    html! {
        h3 {
            a href=(format!("entries/{}",e.id)) {(e.title)}
        }
        p {(e.description)}
    }
}

type Ratings = Vec<(Rating, Vec<Comment>)>;

pub struct EntryPresenter {
    pub entry: Entry,
    pub ratings: HashMap<RatingContext, Ratings>,
}

impl From<(Entry, Vec<(Rating, Vec<Comment>)>)> for EntryPresenter {
    fn from((entry, rtngs): (Entry, Vec<(Rating, Vec<Comment>)>)) -> EntryPresenter {
        let mut ratings: HashMap<RatingContext, Ratings> = HashMap::new();

        for (r, comments) in rtngs {
            if let Some(x) = ratings.get_mut(&r.context) {
                x.push((r, comments));
            } else {
                ratings.insert(r.context, vec![(r, comments)]);
            }
        }

        EntryPresenter { entry, ratings }
    }
}

pub fn entry(e: EntryPresenter) -> Markup {
    page(
        &format!("{} | OpenFairDB", e.entry.title),
        Some(leaflet_css_link()),
        entry_detail(e),
    )
}

fn entry_detail(e: EntryPresenter) -> Markup {
    html! {
        h3 { (e.entry.title) }
        p {(e.entry.description)}
        p {
            table {
                @if let Some(ref h) = e.entry.homepage {
                    tr {
                        td { "Homepage" }
                        td { a href=(h) { (h) } }
                    }
                }
                @if let Some(ref c) = e.entry.contact {
                    @if let Some(ref m) = c.email {
                        tr {
                            td { "eMail" }
                            td { a href=(format!("mailto:{}",m)) { (m) } }
                        }
                    }
                    @if let Some(ref t) = c.telephone {
                        tr {
                            td { "Telephone" }
                            td { a href=(format!("tel:{}",t)) { (t) } }
                        }
                    }
                }
                @if let Some(ref a) = e.entry.location.address {
                    @if !a.is_empty() {
                        tr {
                            td { "Address" }
                            td { (address_to_html(&a)) }
                        }
                    }
                }
            }
        }
        p {
            ul {
                @for t in &e.entry.tags{
                    li{ (format!("#{}", t)) }
                }
            }
        }
        h3 { "Ratings" }

        @for (ctx, ratings) in e.ratings {
            h4 { (format!("{:?}",ctx)) }
            ul {
                @for (r,comments) in ratings {
                    li {
                        (rating(&r, &comments))
                    }
                }
            }
        }
        div id="map" style="height:300px;" { }
        (map_scripts(&[e.entry.into()]))
    }
}

fn rating(r: &Rating, comments: &[Comment]) -> Markup {
    html! {
      h5 { (r.title) " " span { (format!("({})",i8::from(r.value))) } }
      // TODO:
      // form action = "/rating/delete" method = "POST" {
      //     input type="hidden" name="id" value=(r.id);
      //     input type="submit" value="delete rating";
      // }
      @if let Some(ref src) = r.source {
          p { (format!("source: {}",src)) }
      }
      ul {
          @for c in comments {
              li {
                  p { (c.text) }
                  // TODO:
                  // form action = "/comments/delete" method = "POST" {
                  //     input type="hidden" name="id" value=(c.id);
                  //     input type="submit" value="delete comment";
                  // }
              }
          }
      }
    }
}

fn address_to_html(addr: &Address) -> Markup {
    html! {
        @if let Some(ref s) = addr.street {
            (s) br;
        }
        @if let Some(ref z) = addr.zip {
            (z)
        }
        @if let Some(ref z) = addr.city {
            (z) br;
        }
        @if let Some(ref c) = addr.country {
            (c)
        }
    }
}

pub fn event(ev: Event) -> Markup {
    page(
        &ev.title,
        Some(html! {
            link
                rel="stylesheet"
                href=(LEAFLET_CSS_URL)
                integrity=(LEAFLET_CSS_SHA512)
                crossorigin="anonymous";
        }),
        html! {
            h2{ (ev.title) }
            h4 { "Zeit" }
            p{
                (ev.start.to_string())
                    @if let Some(end) = ev.end{
                        " bis "
                        (end.to_string())
                    }
            }
            h4 { "Beschreibung" }
            p{ (ev.description.unwrap_or("".to_string())) }

            @if let Some(ref location) = ev.location{
                    h4{ "Ort" }
                    @if let Some(ref addr) = location.address {
                        @if !addr.is_empty(){
                            @if let Some(ref s) = addr.street {
                             (s) br;
                            }
                            @if let Some(ref z) = addr.zip {
                             (z) " "
                            }
                            @if let Some(ref c) = addr.city {
                             (c) br;
                            }
                            @if let Some(ref c) = addr.country {
                             (c)
                            }
                        }
                    }
                    h4 {"Koordinaten"}
                    p{
                        (format!("{:.2} / {:.2}",location.pos.lat().to_deg(), location.pos.lng().to_deg()))
                    }
            }
            @if let Some(org) = ev.organizer {
                h4{"Veranstalter"}
                p{(org)}
            }
            @if let Some(contact) = ev.contact{
                @if !contact.is_empty(){
                    h4{ "Kontakt" }
                    @if let Some(email) = contact.email{
                        (email)
                        br;
                    }
                    @if let Some(phone) = contact.telephone{
                        (phone)
                    }
                }
            }
            @if let Some(url) = ev.homepage{
                    h4{ "Webseite" }
                    p{
                        a href=(url) { (url) }
                    }
            }
            @if let Some(reg) = ev.registration{
                h4{ "Anmeldung"}
                p {
                    @match reg{
                        RegistrationType::Email => "eMail" ,
                        RegistrationType::Phone => "Telefon",
                            RegistrationType::Homepage => "Webseite",
                    }
                }
            }
            p {
                h4{"Tags"}
                ul{
                    @for t in ev.tags{
                        li {(t)}
                    }
                }
            }

            @if let Some(l) = ev.location {
                div id="map" style="height:300px;" { }
                (map_scripts(&[l.into()]))
            }
        },
    )
}

struct MapPin {
    lat: f64,
    lng: f64,
}

impl MapPin {
    fn to_js_object_string(&self) -> String {
        format!("{{lat:{},lng:{}}}", self.lat, self.lng)
    }
}

impl From<Entry> for MapPin {
    fn from(e: Entry) -> Self {
        e.location.into()
    }
}

impl From<&Entry> for MapPin {
    fn from(e: &Entry) -> Self {
        (&e.location).into()
    }
}

impl From<Location> for MapPin {
    fn from(l: Location) -> Self {
        (&l).into()
    }
}

impl From<&Location> for MapPin {
    fn from(l: &Location) -> Self {
        MapPin {
            lat: l.pos.lat().to_deg(),
            lng: l.pos.lng().to_deg(),
        }
    }
}

fn map_scripts(pins: &[MapPin]) -> Markup {
    let (center, zoom) = match pins.len() {
        1 => ((pins[0].lat, pins[0].lng), 13.0),
        _ => {
            //TODO: calculate center & zoom
            ((48.720, 9.152), 6.0)
        }
    };

    let center = format!("[{},{}]", center.0, center.1);

    let pins: String = pins
        .iter()
        .map(|p| p.to_js_object_string())
        .collect::<Vec<_>>()
        .join(",");

    html! {
      script{
        (format!("window.OFDB_MAP_PINS=[{}];window.OFDB_MAP_ZOOM={};OFDB_MAP_CENTER={};",
                 pins,
                 zoom,
                 center))
      }
      script
        src=(LEAFLET_JS_URL)
        integrity=(LEAFLET_JS_SHA512)
        crossorigin="anonymous" {}
      script src=(MAP_JS_URL){}
    }
}

pub fn events(events: &[Event]) -> Markup {
    page(
        "List of Events",
        None,
        html! {
            div class="events" {
                h3 { "Events" }
                ul class="event-list" {
                    @for e in events {
                        li {
                            a href=(format!("/events/{}", e.id)) {
                                div {
                                    h4 {
                                        span class="title" { (e.title) }
                                        " "
                                        span class="date" {
                                            (e.start.format("%d.%m.%y"))
                                        }
                                    }
                                    p {
                                        @if let Some(ref l) = e.location {
                                            @if let Some(ref a) = l.address {
                                                @if let Some(ref city) = a.city {
                                                    span class="city" { (city) }
                                                    br;
                                                }
                                            }
                                        }
                                        @if let Some(ref o) = e.organizer {
                                        span class="organizer" { (o) }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn login(flash: Option<FlashMessage>) -> Markup {
    flash_page(
        "Login",
        None,
        flash,
        html! {
          form class="login" action="login" method="POST" {
              fieldset{
                label {
                    "eMail:"
                    br;
                    input type="email" name="email" placeholder="eMail address";
                }
                    br;
                label{
                    "Password:"
                    br;
                    input type="password" name="password" placeholder="Password";
                }
                br;
                input type="submit" value="login";
              }
          }
        },
    )
}

pub fn register(flash: Option<FlashMessage>) -> Markup {
    flash_page(
        "Register",
        None,
        flash,
        html! {
          form class="register" action="register" method="POST" {
              fieldset{
                label {
                    "eMail:"
                    br;
                    input type="email" name="email" placeholder="eMail address";
                }
                    br;
                label{
                    "Password:"
                    br;
                    input type="password" name="password" placeholder="Password";
                }
                br;
                input type="submit" value="register";
              }
          }
        },
    )
}

fn flash_msg(flash: Option<FlashMessage>) -> Markup {
    html! {
        @if let Some(msg) = flash {
            div class=(format!("flash {}", msg.name())) {
                (msg.msg())
            }
        }
    }
}

fn flash_page(
    title: &str,
    h: Option<Markup>,
    flash: Option<FlashMessage>,
    content: Markup,
) -> Markup {
    page(
        title,
        h,
        html! {
          (flash_msg(flash))
          (content)
        },
    )
}

fn page(title: &str, h: Option<Markup>, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        head{
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
            title {(title)}
            link rel="stylesheet" href=(MAIN_CSS_URL);
            @if let Some(h) = h {
               (h)
            }
        }
        body{
            (content)
        }
    }
}
