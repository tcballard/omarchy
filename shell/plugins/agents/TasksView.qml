import QtQuick
import QtQuick.Controls as Controls
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Commons as Commons
import qs.Ui as UI

Column {
  id: root
  property bool active: false
  signal closeRequested()
  spacing: Style.space(12)
  property var tasks: []
  property var selected: null
  property string selectedId: ""
  property int activeCount: 0
  property string notice: ""
  property string providers: ""
  property string setupOutput: ""

  function refresh() { status.send("list", {selectedId: selectedId}) }
  function selectTask(id) { selectedId = id; refresh() }
  function cancelTask(id) { action.send("cancel", {taskId: id}) }
  function focusForm() { project.forceActiveFocus() }
  onActiveChanged: if (active) refresh()
  Keys.onEscapePressed: root.closeRequested()
  BrokerClient {
    id: status
    onCompleted: function(value) {
      root.tasks = value.tasks || []
      root.activeCount = Number(value.active || 0)
      root.selected = value.selected || null
    }
  }
  BrokerClient {
    id: action
    onCompleted: function(value) {
      if (value.providers) {
        root.providers = value.providers.map(function(p) { return p.provider + ": " + (p.installed ? p.version : "not installed") + " · " + p.executionStatus }).join("\n")
      } else if (value.id) root.selectedId = value.id
      root.notice = ""
      root.refresh()
    }
  }
  Timer { interval: 1500; running: root.active && !setup.running; repeat: true; onTriggered: root.refresh() }
  Process {
    id: setup
    command: ["bash", Quickshell.env("OMARCHY_PATH") + "/tools/agent-team/scripts/setup-backend"]
    stdout: StdioCollector { onStreamFinished: root.setupOutput += text }
    stderr: StdioCollector { onStreamFinished: root.setupOutput += text }
    onExited: function(code, status) {
      root.notice = code === 0 ? "Tasks is ready" : "Setup failed. " + root.setupOutput.slice(-2000)
      root.refresh()
    }
  }
  Process {
    id: service
    command: ["systemctl", "--user", "start", "omarchy-agent-team.service"]
    onExited: function(code, status) {
      root.notice = code === 0 ? "Service started" : "Choose Set up Tasks to install or repair the backend."
      root.refresh()
    }
  }

  TeamLabel { width: parent.width; text: "Cross-provider review"; font.pixelSize: Style.font.title; font.bold: true }
  TeamLabel { width: parent.width; text: "Choose a lead. The other provider reviews through MCP.\nExperimental · read-only · Codex + Claude"; opacity: 0.7 }
  TeamLabel { width: parent.width; visible: text !== ""; text: setup.running ? "Setting up Tasks…" : root.notice || action.error || status.error; color: Commons.Color.urgent }
  Flow {
    width: parent.width
    spacing: Style.space(8)
    UI.Button { text: setup.running ? "Setting up…" : "Set up Tasks"; focusable: true; enabled: !setup.running && !service.running && !action.busy; onClicked: { root.setupOutput = ""; root.notice = ""; setup.running = true } }
    UI.Button { text: "Start service"; focusable: true; enabled: !service.running; onClicked: service.running = true }
    UI.Button { text: "Check providers"; focusable: true; enabled: !setup.running && !action.busy; onClicked: action.send("doctor", {}) }
    UI.Button { text: "Refresh"; focusable: true; onClicked: root.refresh() }
  }
  TeamLabel { width: parent.width; text: root.providers; visible: text !== ""; font.pixelSize: Style.font.caption }
  UI.TextField { id: project; width: parent.width; placeholderText: "/home/you/Work/project"; selectByMouse: true; Accessible.name: "Project directory" }
  TeamLabel { width: parent.width; text: "Lead agent"; opacity: 0.7 }
  Flow {
    property string provider: "default"
    id: choices
    width: parent.width
    spacing: Style.space(8)
    Repeater {
      model: ["default", "codex", "claude"]
      delegate: UI.Button {
        required property string modelData
        text: modelData === "default" ? "Omarchy default" : modelData
        selected: choices.provider === modelData
        focusable: true
        onClicked: choices.provider = modelData
      }
    }
  }
  TeamLabel { width: parent.width; text: "Both providers must be installed and signed in. Accounts are pinned when you start. No automatic account or provider fallback."; opacity: 0.7 }
  TeamLabel { width: parent.width; text: "Review brief"; opacity: 0.7 }
  Controls.TextArea {
    id: brief
    width: parent.width
    height: Style.space(100)
    textFormat: TextEdit.PlainText
    text: "Review this project's error handling. Ask the other provider for an independent review, collect its result, then report concrete findings with file references."
    wrapMode: TextEdit.Wrap
    color: Commons.Color.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.body
    selectByMouse: true
    Accessible.name: "Review brief"
  }
  UI.Button {
    width: parent.width
    text: action.busy ? "Starting…" : "Start review"
    focusable: true
    enabled: !setup.running && !action.busy && project.text.trim() !== "" && brief.text.trim() !== ""
    onClicked: action.send("start", {provider: choices.provider, cwd: project.text.trim(), task: brief.text})
  }
  TeamLabel { width: parent.width; text: "Tasks"; font.bold: true }
  TeamLabel { width: parent.width; visible: root.tasks.length === 0; text: status.error ? "Task history unavailable" : "No tasks yet. Choose a project and start a review."; opacity: 0.7 }
  TaskCards {
    width: parent.width
    tasks: root.tasks
    selectedId: root.selectedId
    onPicked: function(id) { root.selectTask(id) }
    onCancelled: function(id) { root.cancelTask(id) }
  }
  TeamLabel { width: parent.width; visible: root.selected !== null; text: root.selected ? root.selected.provider + " · " + root.selected.status : ""; font.bold: true }
  TeamLabel { width: parent.width; visible: root.selected !== null; text: root.selected ? root.selected.error || "" : ""; color: Commons.Color.urgent }
  TeamLabel { width: parent.width; visible: root.selected !== null; text: root.selected ? root.selected.result || "No result yet. " + root.selected.activity : "" }
  TeamLabel { width: parent.width; visible: root.selected && root.selected.output_truncated === true; text: "Result exceeded the display budget and was truncated."; opacity: 0.7 }
  Item { width: 1; height: Style.space(8) }
}
