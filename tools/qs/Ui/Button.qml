import QtQuick
Rectangle {
  id: button
  property string text: ""
  property color foreground: "#cdd6f4"
  property string fontFamily: "monospace"
  property bool bordered: false
  signal clicked()
  color: "transparent"
  border.color: bordered ? Qt.rgba(0.8,0.84,0.96,0.35) : "transparent"
  border.width: bordered ? 1 : 0
  radius: 4
  implicitWidth: label.implicitWidth + 20
  implicitHeight: label.implicitHeight + 12
  Text {
    id: label
    anchors.centerIn: parent
    text: button.text
    color: button.foreground
    font.family: button.fontFamily
    font.pixelSize: 13
  }
}
