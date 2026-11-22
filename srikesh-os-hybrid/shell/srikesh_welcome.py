#!/usr/bin/env python3
"""
SRIKESH OS Hybrid — Modern Animated Welcome / Login Splash Screen
Displays 'SRIKESH OS' and 'Awaken the Power of Technology.' on desktop login,
then cleanly exits after 3.6 seconds (or immediately on click/key press).
"""

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib


CSS = b"""
window {
    background-color: #090E1A;
}
.tile-a {
    background-color: #0EA5E9;
    border-radius: 8px;
}
.tile-b {
    background-color: #38BDF8;
    border-radius: 8px;
}
.tile-c {
    background-color: #6366F1;
    border-radius: 8px;
}
.tile-d {
    background-color: #0284C7;
    border-radius: 8px;
}
.title {
    font-family: "DejaVu Sans", "Liberation Sans", sans-serif;
    font-size: 46px;
    font-weight: 700;
    color: #F8FAFC;
}
.tagline {
    font-family: "DejaVu Sans", "Liberation Sans", sans-serif;
    font-size: 20px;
    font-weight: 400;
    color: #38BDF8;
}
.status {
    font-family: "DejaVu Sans", "Liberation Sans", sans-serif;
    font-size: 13px;
    color: #94A3B8;
}
progressbar trough {
    background-color: #1E293B;
    min-height: 6px;
    border-radius: 3px;
}
progressbar progress {
    background-color: #38BDF8;
    min-height: 6px;
    border-radius: 3px;
}
"""


def make_logo():
    grid = Gtk.Grid()
    grid.set_row_spacing(8)
    grid.set_column_spacing(8)
    grid.set_halign(Gtk.Align.CENTER)
    tiles = [
        ("tile-a", 0, 0),
        ("tile-b", 1, 0),
        ("tile-c", 0, 1),
        ("tile-d", 1, 1),
    ]
    for cls, c, r in tiles:
        box = Gtk.Box()
        box.set_size_request(36, 36)
        box.get_style_context().add_class(cls)
        grid.attach(box, c, r, 1, 1)
    return grid


class WelcomeSplash(Gtk.Window):
    def __init__(self):
        super().__init__(type=Gtk.WindowType.TOPLEVEL)
        self.set_decorated(False)
        self.set_keep_above(True)
        screen = Gdk.Screen.get_default()
        if screen:
            self.set_default_size(screen.get_width(), screen.get_height())
        self.move(0, 0)
        self.fullscreen()

        overlay = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        overlay.set_halign(Gtk.Align.CENTER)
        overlay.set_valign(Gtk.Align.CENTER)

        logo = make_logo()
        overlay.pack_start(logo, False, False, 18)

        title = Gtk.Label(label="SRIKESH OS")
        title.get_style_context().add_class("title")
        overlay.pack_start(title, False, False, 6)

        tagline = Gtk.Label(label="Awaken the Power of Technology.")
        tagline.get_style_context().add_class("tagline")
        overlay.pack_start(tagline, False, False, 10)

        self.progress = Gtk.ProgressBar()
        self.progress.set_size_request(340, 8)
        self.progress.set_halign(Gtk.Align.CENTER)
        self.progress.set_fraction(0.05)
        overlay.pack_start(self.progress, False, False, 18)

        self.status = Gtk.Label(label="Initializing desktop environment...")
        self.status.get_style_context().add_class("status")
        overlay.pack_start(self.status, False, False, 4)

        self.add(overlay)
        self.connect("key-press-event", lambda *_: Gtk.main_quit())
        self.connect("button-press-event", lambda *_: Gtk.main_quit())

        self.step = 0
        self.total_steps = 60  # 60 * 60ms = 3.6 seconds
        GLib.timeout_add(60, self.tick)

    def tick(self):
        self.step += 1
        frac = min(1.0, self.step / self.total_steps)
        self.progress.set_fraction(frac)
        if frac < 0.35:
            self.status.set_text("Initializing SRIKESH OS desktop...")
        elif frac < 0.70:
            self.status.set_text("Starting Wi-Fi & Hardware services...")
        else:
            self.status.set_text("Welcome, Srikesh")
        if self.step >= self.total_steps:
            Gtk.main_quit()
            return False
        return True


def main():
    provider = Gtk.CssProvider()
    provider.load_from_data(CSS)
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )
    w = WelcomeSplash()
    w.show_all()
    Gtk.main()


if __name__ == "__main__":
    main()
