# Déploiement sur Google Cloud

## Introduction

Ce guide explique comment déployer une application Rust sur Google Cloud Run. Cloud Run est un service de calcul sans serveur qui permet d'exécuter des conteneurs sur une plateforme gérée. Il prend en charge les conteneurs Docker et les déploie automatiquement sur une infrastructure sans serveur.

## Prérequis

1. Créez un projet Google Cloud Platform (GCP) en utilisant la console GCP.
2. Installez le SDK Google Cloud (gcloud) sur votre machine locale.
3. Installez Docker sur votre machine locale.
4. Installez Rust sur votre machine locale.

## Initialisation du projet

1. Initialisez le SDK: gcloud init et suivez les instructions pour vous connecter et sélectionnez le projet.
2. Activer les API nécessaires:

```bash
gcloud services enable run.googleapis.com
gcloud services enable cloudbuild.googleapis.com
gcloud services enable containerregistry.googleapis.com
```

3. Configurer l'authentification Docker pour Container Registry:

```bash
gcloud auth configure-docker
```

## Déployer l'Application Rust sur Google Cloud Run

### Créer un fichier Dockerfile

Il est nécessaire de créer un fichier Dockerfile à la racine du projet pour définir l'image Docker.

### Construire l'image Docker et la pousser vers Google Container Registry (GCR):

Remplacez <IMAGE_NAME> par le nom de votre image (par exemple, dyno_code) et <PROJECT_ID> par votre ID de projet Google Cloud.

```bash
gcloud builds submit --tag gcr.io/<PROJECT_ID>/<IMAGE_NAME>
```

### Déployer l'image sur Cloud Run:

Remplacez <SERVICE_NAME> par le nom de votre service Cloud Run (par exemple, dyno-code-service) et <IMAGE_NAME> par le nom de votre image.

```bash
gcloud run deploy <SERVICE_NAME> --image gcr.io/<PROJECT_ID>/<IMAGE_NAME> --platform managed --allow-unauthenticated --region europe-west1
```

`--allow-unauthenticated` permet d'accéder au service sans authentification. Environnement de production, envisagez de gérer l'accès via IAM ou des mécanismes d'authentification.
`--region europe-west1` spécifie la région de déploiement. Choisissez la région la plus proche de vos utilisateurs.

### Déployer sur kube

### Créer le cluster kube

1. Créez un cluster Kubernetes:

````bash
```bash
gcloud container clusters create \
  --machine-type n1-standard-2 \
  --num-nodes 2 \
  --zone us-east1-c \
  --cluster-version latest \
  dyno-code-kube
````

2. Configurez kubectl pour utiliser le cluster:

```bash
  gcloud beta container node-pools create user-pool \
  --machine-type n1-standard-2 \
  --num-nodes 0 \
  --enable-autoscaling \
  --min-nodes 0 \
  --max-nodes 3 \
  --node-labels hub.jupyter.org/node-purpose=user \
  --node-taints hub.jupyter.org_dedicated=user:NoSchedule \
  --zone us-east1-c \
  --cluster dyno-code-kube
```

3. Obtenir les informations d'accès au cluster :

```bash
gcloud container clusters get-credentials dyno-code-kube --zone us-east1-c
```

4. Vérifiez que kubectl est configuré pour utiliser le cluster:

```bash
kubectl config view
kubectl config current-context
cat ~/.kube/config
```

5. Vérifiez que le cluster est prêt:

```bash
kubectl get services
kubectl get service dyno-code-service
```

6. Build l'image Docker et la pousser vers Google Container Registry (GCR):

```bash
docker build -t dyno_code_api/dyno-code:latest .
docker tag dyno_code_api/dyno-code:latest gcr.io/pa2024-421814/dyno-code:latest
docker push gcr.io/pa2024-421814/dyno-code:latest

docker build -f Dockerfile.executor -t gcr.io/pa2024-421814/executor:latest .
docker push gcr.io/pa2024-421814/executor:latest
```

6. Déployez l'application sur le cluster:

```bash
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f role_binding.yaml
```

7. Si vous avez mis a jour le docker latest, vous pouvez mettre a jour l'image du pod:

```bash
kubectl set image deployment/dyno-code-deployment dyno-code=gcr.io/pa2024-421814/dyno-code:latest
```

8. Redeployez l'application:

```bash
kubectl rollout restart deployment/dyno-code-deployment
```

9. Pour voir les logs:

```bash
kubectl logs -f deployment/dyno-code-deployment
```

10. Pour voir les pods:

```bash
 kubectl get pods -l app=dyno-code
```

11. Pour voir les logs d'un pod:

```bash
kubectl logs -f <pod_name>
```

12. Pour les derniers logs:

```bash
kubectl logs -l app=dyno-code --tail=1
```