package tplink

import (
	"net"
)

type SystemInfo struct {
	Mode            string  `json:"active_mode,omitempty"`
	Alias           string  `json:"alias,omitempty"`
	Product         string  `json:"dev_name,omitempty"`
	DeviceId        string  `json:"device_id,omitempty"`
	ErrorCode       int     `json:"err_code,omitempty"`
	Features        string  `json:"feature,omitempty"`
	FirmwareId      string  `json:"fwId,omitempty"`
	HardwareId      string  `json:"hwId,omitempty"`
	HardwareVersion string  `json:"hw_ver,omitempty"`
	IconHash        string  `json:"icon_hash,omitempty"`
	GpsLatitude     float32 `json:"latitude,omitempty"`
	GpsLongitude    float32 `json:"longitude,omitempty"`
	LedOff          uint8   `json:"led_off,omitempty"`
	Mac             string  `json:"mac,omitempty"`
	Model           string  `json:"model,omitempty"`
	OemId           string  `json:"odemId,omitempty"`
	OnTime          uint32  `json:"on_time,omitempty"`
	RelayOn         uint8   `json:"relay_state,omitempty"`
	Rssi            int     `json:"rssi,omitempty"`
	SoftwareVersion string  `json:"sw_ver,omitempty"`
	ProductType     string  `json:"type,omitempty"`
	Updating        uint8   `json:"updating,omitempty"`
}

type System struct {
	Info SystemInfo `json:"get_sysinfo"`
}

type RealTimeEnergy struct {
	ErrorCode          uint8   `json:"err_code,omitempty"`
	PowerMilliWatt     float32 `json:"power_mw,omitempty"`
	VoltageMilliVolt   float32 `json:"voltage_mv,omitempty"`
	CurrentMilliAmpere float32 `json:"current_ma,omitempty"`
	TotalWattHour      float32 `json:"total_wh,omitempty"`
}

type EMeter struct {
	RealTime RealTimeEnergy `json:"get_realtime"`
}

type DeviceInfoRequest struct {
	System System `json:"system"`
	EMeter EMeter `json:"emeter"`
}

type DeviceInfoResponse struct {
	System *System `json:"system,omitempty"`
	EMeter *EMeter `json:"emeter,omitempty"`
}

type Device struct {
	Addr *net.UDPAddr
	Data []byte
	Info *DeviceInfoResponse
}
