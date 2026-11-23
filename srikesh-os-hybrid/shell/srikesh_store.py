#!/usr/bin/env python3
"""
SRIKESH OS Hybrid — Native Graphical Snap Store (GTK3)
Backed directly by real Canonical snapd (`snap find`, `snap install`, `snap list`, `snap info`, `snap remove`).
"""

import gi
gi.require_version("Gtk", "3.0")
gi.require_version("Gdk", "3.0")
from gi.repository import Gtk, Gdk, GLib

import os
import subprocess
import threading


STORE_CSS = b"""
window, .store-bg {
    background-color: #111827;
    color: #F1F5F9;
    font-family: "DejaVu Sans", "Liberation Sans", sans-serif;
}
button {
    background-image: none;
    box-shadow: none;
    text-shadow: none;
}
.store-header {
    background-color: #0F172A;
    border-bottom: 1px solid #2A3447;
    padding: 12px 16px;
}
.store-title {
    font-size: 18px;
    font-weight: 700;
    color: #F8FAFC;
}
.store-sub {
    font-size: 11px;
    color: #94A3B8;
}
.badge-ok {
    background-color: #064E3B;
    color: #34D399;
    border: 1px solid #10B981;
    border-radius: 12px;
    padding: 3px 10px;
    font-size: 11px;
    font-weight: 700;
}
.badge-warn {
    background-color: #78350F;
    color: #FBBF24;
    border: 1px solid #F59E0B;
    border-radius: 12px;
    padding: 3px 10px;
    font-size: 11px;
    font-weight: 700;
}
.search-entry {
    background-color: #1E293B;
    color: #F8FAFC;
    border: 1px solid #334155;
    border-bottom: 2px solid #38BDF8;
    border-radius: 6px;
    padding: 7px 10px;
    font-size: 13px;
}
.cat-btn {
    background-color: #1E293B;
    color: #CBD5E1;
    border: 1px solid #334155;
    border-radius: 6px;
    padding: 5px 12px;
    font-size: 12px;
    font-weight: 600;
}
.cat-btn:hover, .cat-btn-active {
    background-color: #0284C7;
    color: #FFFFFF;
    border-color: #38BDF8;
}
.app-card {
    background-color: #1E2430;
    border: 1px solid #2D3748;
    border-radius: 8px;
    padding: 10px 14px;
    margin: 4px 12px;
}
.app-card:hover {
    border-color: #38BDF8;
    background-color: #242C3B;
}
.app-name {
    font-size: 14px;
    font-weight: 700;
    color: #F8FAFC;
}
.app-meta {
    font-size: 11px;
    color: #38BDF8;
    font-weight: 600;
}
.app-desc {
    font-size: 12px;
    color: #CBD5E1;
}
.btn-install {
    background-color: #0EA5E9;
    color: #FFFFFF;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-weight: 700;
}
.btn-install:hover {
    background-color: #38BDF8;
}
.btn-launch {
    background-color: #059669;
    color: #FFFFFF;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-weight: 700;
}
.btn-launch:hover {
    background-color: #10B981;
}
.btn-remove {
    background-color: #991B1B;
    color: #FFFFFF;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
}
.btn-remove:hover {
    background-color: #DC2626;
}
.btn-secondary {
    background-color: #334155;
    color: #F1F5F9;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 600;
}
.btn-secondary:hover {
    background-color: #475569;
}
.console-box, textview, textview text {
    background-color: #0B1120;
    color: #E2E8F0;
    font-family: "DejaVu Sans Mono", monospace;
    font-size: 11px;
}
"""


def run_cmd(args, timeout=20):
    try:
        r = subprocess.run(args, capture_output=True, text=True, timeout=timeout)
        return r.returncode, r.stdout, r.stderr
    except Exception as e:
        return 1, "", str(e)


def parse_snap_table(output):
    rows = []
    lines = [ln for ln in output.splitlines() if ln.strip()]
    if len(lines) < 2:
        return rows
    for ln in lines[1:]:
        parts = ln.split(None, 4)
        if len(parts) >= 4:
            name = parts[0]
            version = parts[1]
            publisher = parts[2]
            notes = parts[3]
            summary = parts[4] if len(parts) > 4 else ""
            rows.append({
                "name": name,
                "version": version,
                "publisher": publisher,
                "notes": notes,
                "summary": summary,
            })
    return rows


def get_installed_snaps():
    rc, out, _ = run_cmd(["snap", "list"], timeout=10)
    if rc != 0:
        return {}
    installed = {}
    lines = [ln for ln in out.splitlines() if ln.strip()]
    for ln in lines[1:]:
        parts = ln.split()
        if len(parts) >= 2:
            installed[parts[0]] = {
                "name": parts[0],
                "version": parts[1],
                "rev": parts[2] if len(parts) > 2 else "-",
                "tracking": parts[3] if len(parts) > 3 else "-",
                "publisher": parts[4] if len(parts) > 4 else "-",
                "notes": parts[5] if len(parts) > 5 else "",
            }
    return installed


class SrikeshSnapStore(Gtk.Window):
    def __init__(self):
        super().__init__(title="SRIKESH OS — Snap Store")
        self.set_default_size(860, 620)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.get_style_context().add_class("store-bg")

        self.busy = False
        self.installed = {}
        self.active_cat = "Featured"
        self.cat_buttons = {}

        root = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        self.add(root)

        # Header
        hdr = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        hdr.get_style_context().add_class("store-header")

        title_col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=2)
        t_lbl = Gtk.Label(label="❖  SRIKESH OS  •  Snap Store")
        t_lbl.set_xalign(0.0)
        t_lbl.get_style_context().add_class("store-title")
        s_lbl = Gtk.Label(label="Powered by real Canonical snapd  •  Search, install & launch live Snap packages")
        s_lbl.set_xalign(0.0)
        s_lbl.get_style_context().add_class("store-sub")
        title_col.pack_start(t_lbl, False, False, 0)
        title_col.pack_start(s_lbl, False, False, 0)
        hdr.pack_start(title_col, True, True, 0)

        self.daemon_badge = Gtk.Label(label="Checking snapd...")
        self.daemon_badge.get_style_context().add_class("badge-warn")
        hdr.pack_end(self.daemon_badge, False, False, 0)

        self.start_daemon_btn = Gtk.Button(label="Start snapd")
        self.start_daemon_btn.get_style_context().add_class("btn-secondary")
        self.start_daemon_btn.connect("clicked", self.on_start_snapd)
        self.start_daemon_btn.set_no_show_all(True)
        hdr.pack_end(self.start_daemon_btn, False, False, 4)

        root.pack_start(hdr, False, False, 0)

        # Search + Categories Bar
        ctrl = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        ctrl.set_margin_top(10)
        ctrl.set_margin_bottom(8)
        ctrl.set_margin_start(14)
        ctrl.set_margin_end(14)

        search_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self.search_entry = Gtk.Entry()
        self.search_entry.set_placeholder_text("Search Canonical Snap Store (e.g. vlc, hello-world, htop, gimp, spotify, code)...")
        self.search_entry.get_style_context().add_class("search-entry")
        self.search_entry.connect("activate", self.on_search_triggered)
        search_row.pack_start(self.search_entry, True, True, 0)

        search_btn = Gtk.Button(label="Search Snap Store")
        search_btn.get_style_context().add_class("btn-install")
        search_btn.connect("clicked", self.on_search_triggered)
        search_row.pack_start(search_btn, False, False, 0)

        wifi_btn = Gtk.Button(label="▂▄▆ Wi-Fi")
        wifi_btn.get_style_context().add_class("btn-secondary")
        wifi_btn.connect("clicked", lambda *_: subprocess.Popen(["pkill", "-USR1", "-f", "srikesh_shell.py"]))
        search_row.pack_start(wifi_btn, False, False, 0)

        ctrl.pack_start(search_row, False, False, 0)

        cat_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
        self.categories = [
            ("Featured", "vlc"),
            ("Utilities", "htop"),
            ("Developer", "code"),
            ("Media", "music"),
            ("Games", "game"),
            ("Installed", "__INSTALLED__"),
        ]
        for label, query in self.categories:
            b = Gtk.Button(label=label)
            b.get_style_context().add_class("cat-btn")
            b.connect("clicked", lambda *_, l=label, q=query: self.select_category(l, q))
            self.cat_buttons[label] = b
            cat_row.pack_start(b, False, False, 0)

        ctrl.pack_start(cat_row, False, False, 0)
        root.pack_start(ctrl, False, False, 0)

        # Status line
        self.status_lbl = Gtk.Label(label="Initializing Snap Store...")
        self.status_lbl.set_xalign(0.0)
        self.status_lbl.set_margin_start(14)
        self.status_lbl.set_margin_bottom(4)
        self.status_lbl.get_style_context().add_class("store-sub")
        root.pack_start(self.status_lbl, False, False, 0)

        # App list scroll area
        scroll = Gtk.ScrolledWindow()
        scroll.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.list_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        scroll.add(self.list_box)
        root.pack_start(scroll, True, True, 0)

        # Bottom Live Activity Console
        log_hdr = Gtk.Label(label="Snap Activity Log (`snapd`)")
        log_hdr.set_xalign(0.0)
        log_hdr.set_margin_start(14)
        log_hdr.set_margin_top(6)
        log_hdr.get_style_context().add_class("store-sub")
        root.pack_start(log_hdr, False, False, 2)

        log_scroll = Gtk.ScrolledWindow()
        log_scroll.set_size_request(-1, 115)
        log_scroll.set_margin_start(12)
        log_scroll.set_margin_end(12)
        log_scroll.set_margin_bottom(10)
        self.log_view = Gtk.TextView()
        self.log_view.set_editable(False)
        self.log_view.set_cursor_visible(False)
        self.log_view.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        self.log_view.get_style_context().add_class("console-box")
        self.log_buf = self.log_view.get_buffer()
        log_scroll.add(self.log_view)
        root.pack_start(log_scroll, False, False, 0)

        self.connect("destroy", Gtk.main_quit)
        self.log("SRIKESH OS Snap Store ready. Checking snapd daemon...")
        self.check_daemon_and_load()

    def log(self, text):
        def _append():
            end = self.log_buf.get_end_iter()
            self.log_buf.insert(end, text.rstrip() + "\n")
            mark = self.log_buf.create_mark(None, self.log_buf.get_end_iter(), False)
            self.log_view.scroll_to_mark(mark, 0.0, False, 0.0, 0.0)
        GLib.idle_add(_append)

    def check_daemon_and_load(self):
        def worker():
            rc, out, _ = run_cmd(["systemctl", "is-active", "snapd"], timeout=5)
            active = (rc == 0 and out.strip() == "active")
            if not active:
                # Trigger socket activation cleanly
                run_cmd(["sudo", "systemctl", "start", "snapd.socket", "snapd.service"], timeout=15)
                rc, out, _ = run_cmd(["systemctl", "is-active", "snapd"], timeout=5)
                active = (rc == 0 and out.strip() == "active")

            rc_v, ver_out, _ = run_cmd(["snap", "version"], timeout=8)
            snap_ver = ""
            if rc_v == 0:
                for ln in ver_out.splitlines():
                    if ln.startswith("snapd "):
                        snap_ver = ln.split()[-1]
                        break
            self.installed = get_installed_snaps()
            GLib.idle_add(self._on_daemon_checked, active, snap_ver)

        threading.Thread(target=worker, daemon=True).start()

    def _on_daemon_checked(self, active, snap_ver):
        ctx = self.daemon_badge.get_style_context()
        ctx.remove_class("badge-ok")
        ctx.remove_class("badge-warn")
        if active:
            ctx.add_class("badge-ok")
            ver_txt = f" v{snap_ver}" if snap_ver else ""
            self.daemon_badge.set_text(f"● snapd ACTIVE{ver_txt}")
            self.start_daemon_btn.hide()
            self.log(f"Connected to real Canonical snapd{ver_txt} ({len(self.installed)} snap(s) installed).")
        else:
            ctx.add_class("badge-warn")
            self.daemon_badge.set_text("● snapd INACTIVE")
            self.start_daemon_btn.show()
            self.log("snapd service not active yet — click 'Start snapd' to start.")
        self.select_category("Featured", "hello")

    def on_start_snapd(self, *_):
        self.log("Starting snapd.socket & snapd.service...")
        self.check_daemon_and_load()

    def select_category(self, label, query):
        self.active_cat = label
        for name, btn in self.cat_buttons.items():
            ctx = btn.get_style_context()
            if name == label:
                ctx.add_class("cat-btn-active")
            else:
                ctx.remove_class("cat-btn-active")
        if query == "__INSTALLED__":
            self.load_installed_view()
        else:
            self.run_snap_search(query, label)

    def on_search_triggered(self, *_):
        q = self.search_entry.get_text().strip()
        if not q:
            return
        for btn in self.cat_buttons.values():
            btn.get_style_context().remove_class("cat-btn-active")
        self.run_snap_search(q, f"Search: '{q}'")

    def clear_list(self):
        for c in self.list_box.get_children():
            self.list_box.remove(c)

    def load_installed_view(self):
        self.status_lbl.set_text("Loading installed Snap packages (`snap list`)...")
        def worker():
            self.installed = get_installed_snaps()
            items = []
            for name, meta in self.installed.items():
                items.append({
                    "name": name,
                    "version": meta["version"],
                    "publisher": meta["publisher"],
                    "notes": meta["notes"],
                    "summary": f"Rev {meta['rev']} • Channel: {meta['tracking']}",
                })
            GLib.idle_add(self.render_items, items, "Installed Snaps", "")
        threading.Thread(target=worker, daemon=True).start()

    def run_snap_search(self, query, title):
        self.status_lbl.set_text(f"Querying Canonical Snap Store (`snap find {query}`)...")
        self.log(f"$ snap find {query}")
        def worker():
            self.installed = get_installed_snaps()
            rc, out, err = run_cmd(["snap", "find", query], timeout=25)
            if rc != 0:
                GLib.idle_add(self.render_items, [], title, err.strip() or out.strip() or "Network offline or Snap Store unreachable.")
                return
            items = parse_snap_table(out)[:24]
            GLib.idle_add(self.render_items, items, title, "")
        threading.Thread(target=worker, daemon=True).start()

    def render_items(self, items, title, error_msg):
        self.clear_list()
        if error_msg:
            self.status_lbl.set_text(f"{title} — offline / unreachable")
            box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
            box.get_style_context().add_class("app-card")
            l1 = Gtk.Label(label="Unable to reach Canonical Snap Store")
            l1.set_xalign(0.0)
            l1.get_style_context().add_class("app-name")
            l2 = Gtk.Label(label=f"Connect to Wi-Fi or Ethernet first, then retry.\nDetails: {error_msg[:180]}")
            l2.set_xalign(0.0)
            l2.set_line_wrap(True)
            l2.get_style_context().add_class("app-desc")
            box.pack_start(l1, False, False, 0)
            box.pack_start(l2, False, False, 0)
            self.list_box.pack_start(box, False, False, 6)
            self.list_box.show_all()
            return

        if not items:
            self.status_lbl.set_text(f"{title} — 0 packages found")
            lbl = Gtk.Label(label="No Snap packages found for this view. Try searching for 'vlc', 'hello-world', 'htop', or 'gimp'.")
            lbl.set_margin_top(24)
            lbl.get_style_context().add_class("app-desc")
            self.list_box.pack_start(lbl, False, False, 0)
            self.list_box.show_all()
            return

        self.status_lbl.set_text(f"{title} — showing {len(items)} live Canonical Snap Store package(s)")
        for item in items:
            name = item["name"]
            is_inst = name in self.installed
            card = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
            card.get_style_context().add_class("app-card")

            info_col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=3)
            top_line = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
            n_lbl = Gtk.Label(label=name)
            n_lbl.set_xalign(0.0)
            n_lbl.get_style_context().add_class("app-name")
            m_lbl = Gtk.Label(label=f"v{item['version']}  •  {item['publisher']}")
            m_lbl.set_xalign(0.0)
            m_lbl.get_style_context().add_class("app-meta")
            top_line.pack_start(n_lbl, False, False, 0)
            top_line.pack_start(m_lbl, False, False, 0)
            info_col.pack_start(top_line, False, False, 0)

            d_lbl = Gtk.Label(label=item["summary"] or "Canonical Snap package")
            d_lbl.set_xalign(0.0)
            d_lbl.set_line_wrap(True)
            d_lbl.get_style_context().add_class("app-desc")
            info_col.pack_start(d_lbl, False, False, 0)
            card.pack_start(info_col, True, True, 0)

            act_col = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=6)
            act_col.set_valign(Gtk.Align.CENTER)

            if is_inst:
                if os.path.exists(f"/snap/bin/{name}"):
                    run_btn = Gtk.Button(label="▶ Launch")
                    run_btn.get_style_context().add_class("btn-launch")
                    run_btn.connect("clicked", lambda *_, p=name: self.launch_snap(p))
                    act_col.pack_start(run_btn, False, False, 0)
                rem_btn = Gtk.Button(label="Remove")
                rem_btn.get_style_context().add_class("btn-remove")
                rem_btn.connect("clicked", lambda *_, p=name: self.remove_snap(p))
                act_col.pack_start(rem_btn, False, False, 0)
            else:
                classic = ("classic" in (item.get("notes") or "").lower())
                inst_btn = Gtk.Button(label="⬇ Install")
                inst_btn.get_style_context().add_class("btn-install")
                inst_btn.connect("clicked", lambda *_, p=name, cl=classic: self.install_snap(p, cl))
                act_col.pack_start(inst_btn, False, False, 0)

            card.pack_end(act_col, False, False, 0)
            self.list_box.pack_start(card, False, False, 2)

        self.list_box.show_all()

    def launch_snap(self, name):
        path = f"/snap/bin/{name}"
        self.log(f"Launching {path}...")
        try:
            subprocess.Popen([path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except Exception as e:
            self.log(f"Launch error: {e}")

    def install_snap(self, name, classic=False):
        if self.busy:
            self.log("Another snap operation is already running — please wait.")
            return
        self.busy = True
        cmd = ["sudo", "snap", "install", name]
        if classic:
            cmd.append("--classic")
        self.log(f"$ {' '.join(cmd)}")
        self.status_lbl.set_text(f"Installing '{name}' from Canonical Snap Store...")

        def worker():
            try:
                proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                for line in proc.stdout:
                    self.log(line)
                rc = proc.wait()
                if rc == 0:
                    self.log(f"✓ Successfully installed snap '{name}'!")
                else:
                    self.log(f"✗ Failed to install '{name}' (exit {rc}).")
            except Exception as e:
                self.log(f"✗ Install exception: {e}")
            finally:
                self.busy = False
                GLib.idle_add(self.load_installed_view)

        threading.Thread(target=worker, daemon=True).start()

    def remove_snap(self, name):
        if self.busy:
            self.log("Another snap operation is already running — please wait.")
            return
        self.busy = True
        self.log(f"$ sudo snap remove {name}")
        self.status_lbl.set_text(f"Removing '{name}'...")

        def worker():
            try:
                proc = subprocess.Popen(["sudo", "snap", "remove", name], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
                for line in proc.stdout:
                    self.log(line)
                rc = proc.wait()
                if rc == 0:
                    self.log(f"✓ Removed snap '{name}'.")
                else:
                    self.log(f"✗ Failed to remove '{name}' (exit {rc}).")
            except Exception as e:
                self.log(f"✗ Remove exception: {e}")
            finally:
                self.busy = False
                GLib.idle_add(self.load_installed_view)

        threading.Thread(target=worker, daemon=True).start()


def main():
    settings = Gtk.Settings.get_default()
    if settings:
        settings.set_property("gtk-application-prefer-dark-theme", True)
    provider = Gtk.CssProvider()
    provider.load_from_data(STORE_CSS)
    Gtk.StyleContext.add_provider_for_screen(
        Gdk.Screen.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )
    win = SrikeshSnapStore()
    win.show_all()
    Gtk.main()


if __name__ == "__main__":
    main()
