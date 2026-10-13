import QtQuick
import QtQuick.Window
import Quickshell
import qs.Ui
import qs.Commons
import qs.Commons as Commons

ShellRoot {
  id: root
  property int step: 0
  property int passed: 0

  function check(condition, label) {
    if (!condition) { console.error("FAIL: " + label); Qt.quit(); throw new Error(label) }
    passed++
    console.log("PASS: " + label)
  }

  Window {
    id: window
    visible: true
    width: 760
    height: 260
    color: "#dce2ea"

    Item {
      id: stage
      anchors.fill: parent

      PanelSurface {
        id: plain
        x: 24; y: 28; width: 340; height: 200
        radius: 12; padding: 16
        color: "#f5f6f8"
        borderSpec: Border.flat("#0067b8", 2)
        headerItem: plainHero
        joinedHeader: false

        Item {
          id: plainHero
          x: 18; y: 18; width: 304; height: 40
          Text { text: "NETWORK"; font.pixelSize: 16; font.bold: true; color: "#202124" }
          Text { y: 24; text: "Connected"; font.pixelSize: 12; color: "#62666c" }
        }
        Text { x: 18; y: 88; text: "Wi-Fi networks"; font.pixelSize: 14; color: "#202124" }
      }

      PanelSurface {
        id: surface
        x: 396; y: 28; width: 340; height: 200
        radius: 12; padding: 16
        color: "#f5f6f8"
        borderSpec: Border.flat("#0067b8", 2)
        headerItem: hero

        Item {
          id: hero
          x: 18; y: 18; width: 304; height: 40
          Text { text: "NETWORK"; font.pixelSize: 16; font.bold: true; color: "#202124" }
          Text { y: 24; text: "Connected"; font.pixelSize: 12; color: "#62666c" }
        }
        Text { x: 18; y: 88; text: "Wi-Fi networks"; font.pixelSize: 14; color: "#202124" }
      }
    }
  }

  Timer {
    interval: 100
    running: true
    repeat: true
    onTriggered: {
      switch (root.step++) {
        case 0:
          Commons.Color.loadUserShell("")
          Commons.Color.loadShell('[popups]\nheader-style = "joined"\nheader-background = "#e9edf2"\n')
          break
        case 1:
          root.check(surface.joinedHeader && surface.headerFillVisible, "theme opts into joined header")
          root.check(surface.headerFillHeight === 66, "fill ends below header without moving body")
          root.check(surface.paintBorder === false && plain.paintBorder === true, "border paints once")
          Commons.Color.loadShell('[popups]\nheader-style = "unknown"\n')
          break
        case 2:
          root.check(!surface.joinedHeader, "unknown styles retain inset header")
          Commons.Color.loadShell("")
          break
        case 3:
          root.check(!surface.joinedHeader, "theme switch removes opt-in")
          Commons.Color.loadShell('[popups]\nheader-style = "joined"\nheader-background = "#e9edf2"\n')
          Commons.Color.loadUserShell('[popups]\nheader-style = "inset"\n')
          break
        case 4:
          root.check(!surface.joinedHeader, "user override wins")
          Commons.Color.loadUserShell("")
          hero.visible = false
          break
        case 5:
          root.check(surface.joinedHeader && !surface.headerFillVisible, "hidden header removes fill")
          hero.visible = true
          hero.y = -100
          break
        case 6:
          root.check(surface.headerFillHeight === 0, "scrolled header removes fill")
          hero.y = 18
          hero.height = 500
          break
        case 7:
          root.check(surface.headerFillHeight === 198, "tall header clamps inside frame")
          surface.headerItem = null
          break
        case 8:
          root.check(!surface.headerFillVisible, "missing header removes fill")
          surface.headerItem = hero
          hero.height = 40
          surface.borderSpec = Border.flat("#0067b8", "4 6 8 10")
          break
        case 9:
          root.check(surface.borderLeft === 10 && surface.borderRight === 6, "per-side border widths preserved")
          surface.borderSpec = Border.flat("#0067b8", 2)
          break
        case 10:
          running = false
          stage.grabToImage(function(result) {
            var target = Quickshell.env("PANEL_HEADER_SCREENSHOT")
            if (target) root.check(result.saveToFile(target), "rendered comparison saved")
            console.log("PANEL_HEADER_TESTS_PASSED=" + root.passed)
            Qt.quit()
          })
          break
      }
    }
  }
}
