package main

import (
	"embed"
	"fmt"
	"log"
	"os"

	"gocv.io/x/gocv"

	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
)

//go:embed all:frontend/dist
var assets embed.FS

var version = "dev"

func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--smoke-test" {
			runSmokeTest()
		}
	}

	app, err := NewApp()
	if err != nil {
		log.Fatal(err)
	}

	err = wails.Run(&options.App{
		Title:         "open-camera-mouse",
		Width:         420,
		Height:        820,
		DisableResize: true,
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 27, G: 38, B: 54, A: 1},
		OnStartup:        app.startup,
		OnShutdown:       app.shutdown,
		Bind: []interface{}{
			app,
		},
	})

	if err != nil {
		log.Printf("Error: %v", err)
	}
}

func runSmokeTest() {
	fmt.Printf("open-camera-mouse %s\n", version)
	fmt.Printf("OpenCV %s\n", gocv.OpenCVVersion())
	mat := gocv.NewMat()
	mat.Close()
	fmt.Println("smoke test passed")
	os.Exit(0)
}
