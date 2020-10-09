## Installation

To install this application using Helm run the following commands: 

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace jarvis-tp-link-hs-110-exporter

helm upgrade \
  jarvis-tp-link-hs-110-exporter \
  jorritsalverda/jarvis-tp-link-hs-110-exporter \
  --install \
  --namespace jarvis-tp-link-hs-110-exporter \
  --set secret.gcpServiceAccountKeyfile='{abc: blabla}' \
  --wait
```
