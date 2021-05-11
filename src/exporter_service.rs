use std::error::Error;

use crate::bigquery_client::BigqueryClient;
use crate::config_client::ConfigClient;
use crate::hs110_client::HS110Client;
use crate::state_client::StateClient;

pub struct ExporterServiceConfig {
    config_client: ConfigClient,
    bigquery_client: BigqueryClient,
    state_client: StateClient,
    hs110_client: HS110Client,
}

impl ExporterServiceConfig {
    pub fn new(
        config_client: ConfigClient,
        bigquery_client: BigqueryClient,
        state_client: StateClient,
        hs110_client: HS110Client,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            config_client,
            bigquery_client,
            state_client,
            hs110_client,
        })
    }
}

pub struct ExporterService {
    config: ExporterServiceConfig,
}

impl ExporterService {
    pub fn new(config: ExporterServiceConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.config_client.read_config_from_file()?;

        self.config.bigquery_client.init_table().await?;

        let last_measurement = self.config.state_client.read_state()?;

        let measurement = self
            .config
            .hs110_client
            .get_measurement(config, last_measurement)?;

        self.config
            .bigquery_client
            .insert_measurement(&measurement)
            .await?;

        self.config.state_client.store_state(&measurement).await?;

        Ok(())
    }
}

// func TestRun(t *testing.T) {
// 	t.Run("ReadsConfigFromFile", func(t *testing.T) {

// 		ctx := context.Background()
// 		ctrl := gomock.NewController(t)
// 		defer ctrl.Finish()

// 		configClient := NewMockConfigClient(ctrl)
// 		bigqueryClient := NewMockBigQueryClient(ctrl)
// 		stateClient := NewMockStateClient(ctrl)
// 		modbusClient := NewMockModbusClient(ctrl)

// 		bigqueryInit := true
// 		bigqueryDataset := "dataset"
// 		bigqueryTable := "table"
// 		config := Config{}
// 		measurement := contractsv1.Measurement{}

// 		service, _ := NewExporterService(configClient, bigqueryClient, stateClient, modbusClient)

// 		configClient.EXPECT().ReadConfigFromFile(ctx, gomock.Any()).Return(config, nil)
// 		bigqueryClient.EXPECT().InitBigqueryTable(ctx, bigqueryDataset, bigqueryTable).Return(nil)
// 		stateClient.EXPECT().ReadState(ctx).Return(nil, nil)
// 		modbusClient.EXPECT().GetMeasurement(ctx, config, nil).Return(measurement, nil)
// 		bigqueryClient.EXPECT().InsertMeasurement(ctx, bigqueryDataset, bigqueryTable, measurement).Return(nil)
// 		stateClient.EXPECT().StoreState(ctx, measurement).Return(nil)

// 		// act
// 		err := service.Run(ctx, bigqueryInit, bigqueryDataset, bigqueryTable)

// 		assert.Nil(t, err)
// 	})
// }
