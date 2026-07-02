package app

import "open-camera-mouse/internal/config"

type commandKind int

const (
	cmdPickPoint commandKind = iota
	cmdRecenter
	cmdSetParams
	cmdSetTrackingEnabled
	cmdResetMouse
)

type command struct {
	kind    commandKind
	x, y   int
	params  config.Params
	enabled bool
}
