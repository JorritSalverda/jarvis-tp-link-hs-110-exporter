package tplink

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestEncrypt(t *testing.T) {
	t.Run("ReturnsEncryptedDeviceInfoRequest", func(t *testing.T) {

		client := &client{
			timeout: 2,
		}
		request := DeviceInfoRequest{}
		request_json, err := json.Marshal(&request)
		assert.Nil(t, err)
		assert.Equal(t, "{\"system\":{\"get_sysinfo\":{}},\"emeter\":{\"get_realtime\":{}}}", string(request_json))

		// act
		request_encrypted := client.encrypt(request_json)

		assert.Equal(t, "\xd0\xf2\x81\xf8\x8b\xff\x9a\xf7\xd5\uf536Ѵ\xc0\x9f\xec\x95\xe6\x8f\xe1\x87\xe8\xca\xf0\x8b\xf6\x8b\xa7\x85\xe0\x8d\xe8\x9c\xf9\x8b\xa9\x93\xe8ʭȼ\xe3\x91\xf4\x95\xf9\x8d\xe4\x89\xec\xce\xf4\x8f\xf2\x8f\xf2", string(request_encrypted))
	})
}

func TestDecrypt(t *testing.T) {
	t.Run("ReturnsDecryptedDeviceInfoRequest", func(t *testing.T) {

		client := &client{
			timeout: 2,
		}

		response_encrypted := []byte("\xd0\xf2\x81\xf8\x8b\xff\x9a\xf7\xd5\uf536Ѵ\xc0\x9f\xec\x95\xe6\x8f\xe1\x87\xe8\xca\xf0\x8b\xf6\x8b\xa7\x85\xe0\x8d\xe8\x9c\xf9\x8b\xa9\x93\xe8ʭȼ\xe3\x91\xf4\x95\xf9\x8d\xe4\x89\xec\xce\xf4\x8f\xf2\x8f\xf2")

		// act
		response_json := client.decrypt(response_encrypted)
		assert.Equal(t, "{\"system\":{\"get_sysinfo\":{}},\"emeter\":{\"get_realtime\":{}}}", string(response_json))

		var response DeviceInfoRequest
		err := json.Unmarshal(response_json, &response)
		assert.Nil(t, err)

		assert.Equal(t, DeviceInfoRequest{}, response)
	})
}
