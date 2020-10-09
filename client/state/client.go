package state

import (
	"context"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"

	contractsv1 "github.com/JorritSalverda/jarvis-contracts-golang/contracts/v1"
	"github.com/rs/zerolog/log"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
)

// Client is the interface for retrieving and storing state
type Client interface {
	ReadState(ctx context.Context) (lastMeasurement *contractsv1.Measurement, err error)
	StoreState(ctx context.Context, measurement contractsv1.Measurement) (err error)
}

// NewClient returns new bigquery.Client
func NewClient(ctx context.Context, kubeClientset *kubernetes.Clientset, measurementFilePath, measurementFileConfigMapName string) (Client, error) {

	return &client{
		kubeClientset:                kubeClientset,
		measurementFilePath:          measurementFilePath,
		measurementFileConfigMapName: measurementFileConfigMapName,
	}, nil
}

type client struct {
	kubeClientset                *kubernetes.Clientset
	measurementFilePath          string
	measurementFileConfigMapName string
}

func (c *client) ReadState(ctx context.Context) (lastMeasurement *contractsv1.Measurement, err error) {

	// check if last measurement file exists in configmap
	if _, err := os.Stat(c.measurementFilePath); !os.IsNotExist(err) {
		log.Info().Msgf("File %v exists, reading contents...", c.measurementFilePath)

		// read state file
		data, err := ioutil.ReadFile(c.measurementFilePath)
		if err != nil {
			return lastMeasurement, fmt.Errorf("Failed reading file from path %v: %w", c.measurementFilePath, err)
		}

		log.Info().Msgf("Unmarshalling file %v contents...", c.measurementFilePath)

		// unmarshal state file
		if err := json.Unmarshal(data, &lastMeasurement); err != nil {
			return lastMeasurement, fmt.Errorf("Failed unmarshalling last measurement file: %w", err)
		}
	}

	return
}

func (c *client) StoreState(ctx context.Context, measurement contractsv1.Measurement) (err error) {

	currentNamespace, err := c.getCurrentNamespace()
	if err != nil {
		return
	}

	// retrieve configmap
	configMap, err := c.kubeClientset.CoreV1().ConfigMaps(currentNamespace).Get(ctx, c.measurementFileConfigMapName, metav1.GetOptions{})
	if err != nil {
		return fmt.Errorf("Failed retrieving configmap %v: %w", c.measurementFileConfigMapName, err)
	}

	// marshal state to json
	measurementData, err := json.Marshal(measurement)
	if configMap.Data == nil {
		configMap.Data = make(map[string]string)
	}

	configMap.Data[filepath.Base(c.measurementFilePath)] = string(measurementData)

	// update configmap to have measurement available when the application runs the next time and for other applications
	_, err = c.kubeClientset.CoreV1().ConfigMaps(currentNamespace).Update(ctx, configMap, metav1.UpdateOptions{})
	if err != nil {
		return fmt.Errorf("Failed updating configmap %v: %w", c.measurementFileConfigMapName, err)
	}

	log.Info().Msgf("Stored measurement in configmap %v...", c.measurementFileConfigMapName)

	return nil
}

func (c *client) getCurrentNamespace() (namespace string, err error) {
	ns, err := ioutil.ReadFile("/var/run/secrets/kubernetes.io/serviceaccount/namespace")
	if err != nil {
		return namespace, fmt.Errorf("Failed reading namespace: %w", err)
	}

	return string(ns), nil
}
