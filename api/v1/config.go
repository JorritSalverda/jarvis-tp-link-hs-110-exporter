package api

import (
	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
)

type Config struct {
	Location string `yaml:"location"`

	// default jarvis config for sample
	EntityType contractsv1.EntityType `yaml:"entityType"`
	EntityName string                 `yaml:"entityName"`
}
