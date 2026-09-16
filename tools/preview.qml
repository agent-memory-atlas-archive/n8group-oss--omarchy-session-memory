// Renders the Session Memory panel for the marketplace listing.
//
// The rows, spacing and helper functions are lifted from the real Menu.qml;
// only the data is invented. Nothing from the maintainer's machine appears.
import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Ui

Window {
  id: win
  visible: true
  width: 560
  height: 760
  color: "#11111b"

  property color fg: "#cdd6f4"
  property color dim: "#9399b2"
  property color urgent: "#f38ba8"
  property string fontFamily: "monospace"

    function shortId(id) {
    var value = String(id || "")
    return value.length > 12 ? value.substring(0, 12) + "…" : value
  }
    function placementText(session) {
    var monitor = session.monitor !== undefined && session.monitor !== null && String(session.monitor) !== ""
      ? String(session.monitor)
      : "monitor unknown"
    return monitor
  }
    function goalText(session) {
    var goal = session ? session.goal : null
    if (goal && goal.title) return String(goal.title)
    var rows = (session && session.conversations) ? session.conversations : []
    if (rows.length === 0) return "No conversation was recorded in this session."
    return "untitled"
  }
    function titleText(row) {
    return (row && row.title) ? String(row.title) : "untitled"
  }
    function paneText(row) {
    if (!row) return ""
    var place = "w" + String(row.window_idx) + ".p" + String(row.pane_idx)
    if (row.title_source === "first_prompt") return place + " \u00b7 from the first prompt"
    if (row.title_source === "agent") return place + " \u00b7 agent's own title"
    return place
  }
    function countText(n, one, many) {
    var v = Number(n)
    if (!isFinite(v)) return "unknown"
    return v + " " + (v === 1 ? one : many)
  }
    function resumableNote(total, matching, shown, filter) {
    var t = Number(total)
    var m = Number(matching)
    var s = Number(shown)
    var query = root.normalizedQuery(filter)
    if (!isFinite(t) || !isFinite(m) || !isFinite(s)) return "unknown"
    if (t === 0) return "Nothing waiting to be resumed."
    if (query === "") {
      var head = root.countText(t, "conversation", "conversations") + " waiting to be resumed"
      if (s >= t) return head + "."
      return head + " — showing the " + s + " most recently active."
    }
    var quoted = "“" + String(filter).trim() + "”"
    if (m === 0)
      return "No conversation matches " + quoted + " — "
        + root.countText(t, "conversation", "conversations") + " waiting to be resumed."
    var matched = m + " of " + root.countText(t, "conversation", "conversations")
      + (m === 1 ? " matches " : " match ") + quoted
    if (s >= m) return matched + "."
    return matched + " — showing the " + s + " most recently active."
  }

  readonly property var sessions: [
    { name: "api",     windows: 2, panes: 6, agents: 3, workspace: "2", monitor: "DP-1",
      goal: { title: "Rate limiting for the public endpoints" } },
    { name: "docs",    windows: 1, panes: 2, agents: 1, workspace: "3", monitor: "DP-1",
      goal: { title: "Rewrite the getting-started guide" } },
    { name: "infra",   windows: 1, panes: 4, agents: 2, workspace: "5", monitor: "HDMI-A-1",
      goal: { title: "Migrate the build cache to the new runner" } },
    { name: "scratch", windows: 1, panes: 1, agents: 0, workspace: null, monitor: null,
      goal: null }
  ]

  Rectangle {
    id: card
    width: 560
    height: 760
    color: "#11111b"

  Column {
    anchors.fill: parent
    anchors.margins: 22
    spacing: 14

    Text { text: "Session memory"; color: win.fg; font.family: win.fontFamily; font.pixelSize: 19 }
    Text { text: "OSM 0.1.0"; color: win.dim; font.family: win.fontFamily; font.pixelSize: 12 }
    Text { text: "The engine is running."; color: win.dim; font.family: win.fontFamily; font.pixelSize: 13 }

    Item { width: 1; height: 4 }

    Text { text: "NEWEST SNAPSHOT"; color: win.dim; font.family: win.fontFamily; font.pixelSize: 11 }
    Text {
      text: "#1482 · 40s ago · complete · 4 sessions"
      color: win.fg; font.family: win.fontFamily; font.pixelSize: 13
    }
    Text {
      text: "captures are fresh"
      color: win.dim; font.family: win.fontFamily; font.pixelSize: 11
    }

    Item { width: 1; height: 6 }

    Rectangle {
      width: parent.width; height: 34; radius: 4
      color: "transparent"; border.color: Qt.rgba(0.8,0.84,0.96,0.28); border.width: 1
      Text {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left; anchors.leftMargin: 10
        text: "What it was about, session name, project, kind or id"
        color: win.dim; font.family: win.fontFamily; font.pixelSize: 12
      }
    }
    Text {
      text: "Press / to type here. Escape clears it, then gives the keys back to the panel."
      color: win.dim; font.family: win.fontFamily; font.pixelSize: 11
      width: parent.width; wrapMode: Text.WordWrap
    }

    Item { width: 1; height: 6 }

    Text { text: "RECORDED SESSIONS"; color: win.dim; font.family: win.fontFamily; font.pixelSize: 11 }

    Repeater {
      model: win.sessions
      Column {
        required property var modelData
        width: parent.width
        spacing: 3
        Item { width: 1; height: 7 }
        Text {
          text: modelData.workspace ? ("WORKSPACE " + modelData.workspace) : "NO WINDOW RECORDED"
          color: win.dim; font.family: win.fontFamily; font.pixelSize: 11
        }
        RowLayout {
          width: parent.width
          spacing: 8
          Text {
            Layout.fillWidth: true
            Layout.preferredWidth: 0
            Layout.minimumWidth: Math.min(96, parent.width)
            Layout.horizontalStretchFactor: 1
            elide: Text.ElideRight
            text: modelData.name
            color: win.fg; font.family: win.fontFamily; font.pixelSize: 13
          }
          Text {
            Layout.fillWidth: true
            Layout.horizontalStretchFactor: 0
            horizontalAlignment: Text.AlignRight
            elide: Text.ElideRight
            text: modelData.windows + "w · " + modelData.panes + "p · " + modelData.agents + "a · "
              + (modelData.monitor ? modelData.monitor : "monitor unknown")
            color: win.dim; font.family: win.fontFamily; font.pixelSize: 11
          }
        }
        RowLayout {
          width: parent.width
          spacing: 8
          Text {
            Layout.fillWidth: true
            Layout.preferredWidth: 0
            elide: Text.ElideRight
            text: modelData.goal ? modelData.goal.title : "untitled"
            color: modelData.goal ? win.fg : win.dim
            font.family: win.fontFamily; font.pixelSize: 12
          }
          Button { text: "Detail"; foreground: win.fg; fontFamily: win.fontFamily; bordered: true }
        }
      }
    }
  }

  }

  Timer { interval: 8000; running: true; onTriggered: { console.log("grab timed out"); Qt.exit(2) } }

  Component.onCompleted: {
    Qt.callLater(function () {
      card.grabToImage(function (r) {
        r.saveToFile("preview.png")
        Qt.exit(0)
      }, Qt.size(card.width, card.height))
    })
  }
}
