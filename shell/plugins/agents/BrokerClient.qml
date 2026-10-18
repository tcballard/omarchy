import QtQuick
import Quickshell
import Quickshell.Io

QtObject {
  id: root
  property bool busy: process.running
  property string command: Quickshell.env("HOME") + "/.local/bin/omarchy-agent-team"
  property var reply: null
  property string error: ""
  signal completed(var value)
  function send(op, params) {
    if (busy) return false
    error = ""
    reply = null
    process.command = [command, "request"]
    process.pending = JSON.stringify({op: op, params: params || {}}) + "\n"
    process.running = true
    return true
  }
  property Process process: Process {
    id: process
    property string pending: ""
    stdinEnabled: true
    onStarted: { write(pending); stdinEnabled = false; deadline.restart() }
    stdout: StdioCollector { onStreamFinished: {
      try {
        var result = JSON.parse(text)
        if (result.ok) { root.reply = result.data; root.completed(result.data) }
        else root.error = String(result.error || "Request failed")
      } catch (e) { root.error = "Could not read broker response. Retry or restart the service." }
    } }
    onRunningChanged: if (running) stdinEnabled = true
    onExited: function(code, status) {
      deadline.stop()
      if (code !== 0 && root.error === "") root.error = "Agent Team is unavailable. Choose Set up Tasks to install or repair the backend."
    }
  }
  property Timer deadline: Timer {
    interval: 13000
    onTriggered: { root.error = "Broker request timed out. Retry."; process.signal(15) }
  }
}
