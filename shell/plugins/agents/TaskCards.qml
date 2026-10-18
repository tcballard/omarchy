import QtQuick
import qs.Commons
import qs.Commons as Commons
import qs.Ui as UI

Column {
  id: root
  property var tasks: []
  property string selectedId: ""
  signal picked(string id)
  signal cancelled(string id)
  spacing: Style.space(8)
  Repeater {
    model: root.tasks
    delegate: Column {
      id: card
      required property var modelData
      width: root.width
      spacing: Style.space(4)
      UI.Button {
        width: parent.width
        text: (card.modelData.parent ? "↳ " : "") + card.modelData.provider + " · " + card.modelData.status
        selected: root.selectedId === card.modelData.id
        focusable: true
        onClicked: root.picked(card.modelData.id)
      }
      TeamLabel { width: parent.width; text: card.modelData.prompt; maximumLineCount: 2; elide: Text.ElideRight }
      TeamLabel { width: parent.width; text: card.modelData.activity; opacity: 0.65; font.pixelSize: Style.font.caption }
      UI.Button {
        visible: ["queued", "running", "waiting_for_children"].indexOf(card.modelData.status) >= 0
        text: "Cancel"
        focusable: true
        onClicked: root.cancelled(card.modelData.id)
      }
    }
  }
}
