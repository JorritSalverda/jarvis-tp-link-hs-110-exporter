package config

import (
	"context"
	"testing"

	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	"github.com/stretchr/testify/assert"
)

func TestReadConfigFromFile(t *testing.T) {

	t.Run("ReturnsConfig", func(t *testing.T) {

		ctx := context.Background()
		client, _ := NewClient(ctx)

		// act
		config, err := client.ReadConfigFromFile(ctx, "./test-config.yaml")

		assert.Nil(t, err)
		assert.Equal(t, "My Home", config.Location)
		assert.Equal(t, contractsv1.EntityType_ENTITY_TYPE_DEVICE, config.EntityType)
		assert.Equal(t, "TP-Link HS110", config.EntityName)
	})
}
