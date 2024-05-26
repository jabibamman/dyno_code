#!/bin/sh

docker build -t dyno_code_api/dyno-code:latest .
docker tag dyno_code_api/dyno-code:latest gcr.io/pa2024-421814/dyno-code:latest
docker push gcr.io/pa2024-421814/dyno-code:latest

docker build -f Dockerfile.executor -t gcr.io/pa2024-421814/executor:latest .
docker push gcr.io/pa2024-421814/executor:latest

kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f role_binding.yaml

kubectl set image deployment/dyno-code-deployment dyno-code=gcr.io/pa2024-421814/dyno-code:latest

kubectl rollout restart deployment/dyno-code-deployment
