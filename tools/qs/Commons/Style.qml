pragma Singleton
import QtQml
QtObject {
  function space(n) { return n }
  readonly property QtObject font: QtObject {
    readonly property int body: 14
    readonly property int bodySmall: 13
    readonly property int caption: 11
  }
  readonly property QtObject spacing: QtObject {
    readonly property int labelGap: 4
    readonly property int controlGap: 6
    readonly property int controlPaddingX: 10
    readonly property int controlPaddingY: 6
  }
  readonly property int cornerRadius: 4
}
