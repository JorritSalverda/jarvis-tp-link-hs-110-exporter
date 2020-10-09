package tplink

import (
	"bufio"
	"context"
	"encoding/binary"
	"encoding/json"
	"io"
	"net"
	"strconv"
	"sync"
	"time"

	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	apiv1 "github.com/JorritSalverda/jarvis-tp-link-hs-110-exporter/api/v1"
	"github.com/google/uuid"
	"github.com/rs/zerolog/log"
)

const (
	headerLength = 4
)

// Client is the interface for connecting to a modbus device via ethernet
type Client interface {
	GetMeasurement(ctx context.Context, config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error)
}

// NewClient returns new modbus.Client
func NewClient(ctx context.Context, timeout int) (Client, error) {

	return &client{
		timeout: timeout,
	}, nil
}

type client struct {
	timeout int
}

func (c *client) GetMeasurement(ctx context.Context, config apiv1.Config, lastMeasurement *contractsv1.Measurement) (measurement contractsv1.Measurement, err error) {

	measurement = contractsv1.Measurement{
		ID:             uuid.New().String(),
		Source:         "jarvis-tp-link-hs-110-exporter",
		Location:       config.Location,
		Samples:        []*contractsv1.Sample{},
		MeasuredAtTime: time.Now().UTC(),
	}

	log.Info().Msg("Discovering devices...")
	devices, err := c.discoverDevices(c.timeout)
	if err != nil {
		log.Warn().Err(err).Msg("Failed discovering devices")
	} else {
		log.Info().Interface("devices", devices).Msg("Retrieved devices...")

		devices, err = c.getUsageForAllDevices(devices, c.timeout)
		if err != nil {
			log.Warn().Err(err).Msg("Failed retrieving metrics for devices")
		} else {

			for _, device := range devices {
				// init sample from config
				sample := contractsv1.Sample{
					EntityType: config.EntityType,
					EntityName: config.EntityName,
					SampleType: config.SampleType,
					SampleName: device.Info.System.Info.Alias,
					MetricType: config.MetricType,
				}

				sample.Value = float64(device.Info.EMeter.RealTime.TotalWattHour) * config.ValueMultiplier

				measurement.Samples = append(measurement.Samples, &sample)
			}
		}
	}

	if lastMeasurement != nil {
		measurement.Samples = c.sanitizeSamples(measurement.Samples, lastMeasurement.Samples)
	}

	return
}

func (c *client) discoverDevices(timeout int) (devices []Device, err error) {

	request := DeviceInfoRequest{}

	broadcastAddr, err := net.ResolveUDPAddr("udp", "255.255.255.255:9999")
	if err != nil {
		return
	}

	fromAddr, err := net.ResolveUDPAddr("udp", "0.0.0.0:8755")
	if err != nil {
		return
	}

	sock, err := net.ListenUDP("udp", fromAddr)
	defer sock.Close()
	if err != nil {
		return
	}
	sock.SetReadBuffer(2048)

	r := make(chan Device)

	go func(s *net.UDPConn, request interface{}) {
		for {
			buff := make([]byte, 2048)
			rlen, addr, err := s.ReadFromUDP(buff)
			if err != nil {
				break
			}
			go c.discovered(r, addr, rlen, buff)
		}
	}(sock, request)

	eJSON, err := json.Marshal(&request)
	if err != nil {
		return
	}

	var encrypted = c.encrypt(eJSON)
	_, err = sock.WriteToUDP(encrypted, broadcastAddr)
	if err != nil {
		return
	}
	started := time.Now()
Q:
	for {
		select {
		case x := <-r:

			info := DeviceInfoResponse{}
			json.Unmarshal(x.Data, &info)

			x.Info = &info

			devices = append(devices, x)
		default:
			if now := time.Now(); now.Sub(started) >= time.Duration(timeout)*time.Second {
				break Q
			}
		}
	}
	return devices, nil
}

func (c *client) getUsageForAllDevices(devices []Device, timeout int) (updatedDevices []Device, err error) {

	var wg sync.WaitGroup
	wg.Add(len(devices))

	devicesChannel := make(chan Device, len(devices))
	errors := make(chan error, len(devices))

	for _, d := range devices {
		go func(d Device) {
			defer wg.Done()
			d, err := c.getUsageForDevice(d, timeout)
			if err != nil {
				errors <- err
				return
			}
			devicesChannel <- d
		}(d)
	}

	wg.Wait()

	close(errors)
	for e := range errors {
		return nil, e
	}

	close(devicesChannel)
	for d := range devicesChannel {
		updatedDevices = append(updatedDevices, d)
	}

	return updatedDevices, nil
}

func (c *client) getUsageForDevice(device Device, timeout int) (Device, error) {

	request := DeviceInfoRequest{}

	eJSON, err := json.Marshal(&request)
	if err != nil {
		return device, err
	}

	response, err := c.sendCommand(device.Addr.IP.String(), device.Addr.Port, eJSON, timeout)
	if err != nil {
		return device, err
	}

	err = json.Unmarshal([]byte(response), &device.Info)
	if err != nil {
		return device, err
	}

	return device, nil
}

func (c *client) sanitizeSamples(currentSamples, lastSamples []*contractsv1.Sample) (sanitizeSamples []*contractsv1.Sample) {

	sanitizeSamples = []*contractsv1.Sample{}
	for _, cs := range currentSamples {
		// check if there's a corresponding sample in lastSamples and see if the difference with it's value isn't too large
		sanitize := false
		for _, ls := range lastSamples {
			if cs.EntityType == ls.EntityType &&
				cs.EntityName == ls.EntityName &&
				cs.SampleType == ls.SampleType &&
				cs.SampleName == ls.SampleName &&
				cs.MetricType == cs.MetricType {
				if cs.MetricType == contractsv1.MetricType_METRIC_TYPE_COUNTER && cs.Value/ls.Value > 1.1 {
					log.Warn().Msgf("Value for %v is more than 10 percent larger than the last sampled value %v, keeping previous value instead", cs, ls.Value)
					sanitizeSamples = append(sanitizeSamples, ls)
				}

				break
			}
		}
		if !sanitize {
			sanitizeSamples = append(sanitizeSamples, cs)
		}
	}

	return
}

// sendCommand is based on https://github.com/jaedle/golang-tplink-hs100
func (c *client) sendCommand(address string, port int, command []byte, timeoutSeconds int) ([]byte, error) {
	conn, err := net.DialTimeout("tcp", address+":"+strconv.Itoa(port), time.Duration(timeoutSeconds)*time.Second)
	if err != nil {
		return nil, err
	}
	defer conn.Close()

	writer := bufio.NewWriter(conn)
	_, err = writer.Write(c.encryptWithHeader(command))
	if err != nil {
		return nil, err
	}
	writer.Flush()

	response, err := c.readHeader(conn)
	if err != nil {
		return nil, err
	}

	payload, err := c.readPayload(conn, c.payloadLength(response))
	if err != nil {
		return nil, err
	}

	return c.decrypt(payload), nil
}

func (c *client) readHeader(conn net.Conn) ([]byte, error) {
	headerReader := io.LimitReader(conn, int64(headerLength))
	var response = make([]byte, headerLength)
	_, err := headerReader.Read(response)
	return response, err
}

func (c *client) readPayload(conn net.Conn, length uint32) ([]byte, error) {
	payloadReader := io.LimitReader(conn, int64(length))
	var payload = make([]byte, length)
	_, err := payloadReader.Read(payload)
	return payload, err
}

func (c *client) payloadLength(header []byte) uint32 {
	payloadLength := binary.BigEndian.Uint32(header)
	return payloadLength
}

const lengthHeader = 4

func (c *client) encrypt(input []byte) []byte {
	s := string(input)

	key := byte(0xAB)
	b := make([]byte, len(s))
	for i := 0; i < len(s); i++ {
		b[i] = s[i] ^ key
		key = b[i]
	}
	return b
}

func (c *client) encryptWithHeader(input []byte) []byte {
	s := string(input)

	lengthPayload := len(s)
	b := make([]byte, lengthHeader+lengthPayload)
	copy(b[:lengthHeader], c.header(lengthPayload))
	copy(b[lengthHeader:], c.encrypt(input))
	return b
}

func (c *client) header(lengthPayload int) []byte {
	h := make([]byte, lengthHeader)
	binary.BigEndian.PutUint32(h, uint32(lengthPayload))
	return h
}

func (c *client) decrypt(b []byte) []byte {
	k := byte(0xAB)
	var newKey byte
	for i := 0; i < len(b); i++ {
		newKey = b[i]
		b[i] = b[i] ^ k
		k = newKey
	}

	return b
}

func (c *client) decryptWithHeader(b []byte) []byte {
	return c.decrypt(c.payload(b))
}

func (c *client) payload(b []byte) []byte {
	return b[lengthHeader:]
}

func (c *client) discovered(r chan Device, addr *net.UDPAddr, rlen int, buff []byte) {
	r <- Device{
		Addr: addr,
		Data: c.decrypt(buff[:rlen]),
	}
}
