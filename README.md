Central repo for holodekk related spikes.

# Roadmap

## Milestone 1 (builder)
	- base (Dockerfile)
	- app layer (.tar)
	- runnable manually with manual overlay mounts

## Milestone 2 (runnable)
	- base (Dockerfile)
	- app (self-contained "executable")
	- runnable as a "binary"

## Milestone 3 (registry)
	- app can be "pushed" to an app registry
	- app can be "pulled" from an app registry

# Backlog

### shim
	- build runc shim from capone source
	- build ruby wrapper for shim
	- allow deps to be injected via ruby wrapper
	- handle events from ruby wrapper
	- build runnable object from wrapper
	- add docker driver
	- add kubernetes driver

### kubernetes integrated
	- run "shim" as a kubernetes service (pod)
	- interact with kubernetes from pod

## Milestone 2 (development orchestrator)

## node org supervisor
	- respond to shim start requests
	- respond to shim lifecycle events
	- store state
	- manage cgroups/users/limits

## node supervisor
	- respond to app requests
