import QtQuick
import Quickshell
import qs.Commons
import qs.Commons as Commons

// Explicit header ownership; painting does not change content or input layout.
BorderSurface {
  id: root

  paintBorder: !headerFillVisible

  property Item headerItem: null
  property bool joinedHeader: Commons.Color.popups.joinedHeader
  property color headerBackground: Commons.Color.popups.headerBackground
  readonly property real headerBottom: {
    headerWatcher.transform
    if (!headerItem || !headerItem.visible) return 0
    return headerItem.mapToItem(root, 0, headerItem.height).y
  }
  readonly property real headerFillHeight: headerItem && headerItem.visible
    ? Math.max(0, Math.min(height - borderBottom, headerBottom + topPadding / 2))
    : 0
  readonly property bool headerFillVisible: joinedHeader && headerItem !== null && headerFillHeight > 0 && width > 0

  TransformWatcher {
    id: headerWatcher
    a: root
    b: root.headerItem
  }

  // Clip the same full-frame rounded shape as the parent to a header band.
  // This preserves top corners even when a scrolled header becomes short.
  Item {
    width: root.width
    height: root.headerFillHeight
    visible: root.headerFillVisible
    clip: true

    Rectangle {
      width: root.width
      height: root.height
      radius: root.radius
      color: root.headerBackground
    }
  }

  // Repaint the frame over the joined fill, including native/uniform specs,
  // so asymmetric widths and gradient borders cannot be covered by it.
  Loader {
    anchors.fill: parent
    active: root.headerFillVisible
    z: 1

    sourceComponent: BorderOverlay {
      anchors.fill: parent
      radius: root.radius
      borderSpec: root.borderSpec
    }
  }
}
