# About

resourcepool(d)/respo(d) - a resource (eg network connected equipment test bench) leasing system

It's a HTTP service where you can request exclusive access to a resource, which is composed of a set of entities with
attributes.
A resource could be a testbench (bench), which is typically used for hardware in the loop testing in software validation, and is composed of one or more devices under test, debugging adaptors and measurement equipment.

Hence, a client would request access to a resource by either specifying its name, or submitting a query describing attributes the resource or its entities posess.
When submitting a query, the result can be any resource matching the query, returning an next available resource without the
need for the client to pick a specific one.

# Summary usage

As resource client, I want to do a HTTP POST /lock?location=myoffice&resource=myresource to request use of an a resource by name.
Upon receiving the message that the resource is locked, the client shall be able to use the returned resource (description)
until the client cuts the connection to the service. At that point the lock is considered implicitly released.

Clients get queued if the lock is not available, and can specify a priority value (to have a client skip the queue over a CI/CD runner),
while the client is waiting, the server sends updates on queue usage of the potential candidate resources.

There can be language specific Client libraries (eg python) to use in test scenario's (eg pytest) that use the physical devices

There is a command line tool (rpclient) to claim maintenance on the test setup

The service maintains an inventory of resources, which have an office location, property tags, freeform dict of details (port numbers etc) persisted to disk.

Devops can do a HTTP POST /inventory with yaml to update the inventory (eg. from A CI/CD pipeline), which returns an HTTP error if it fails to validate.

# Security

Security is not a primary concern. The service is intended to be used on-premises, not exposed to the public internet.

For hardening, TLS with client side certificates would be an option, and each resource could be fitted with a local
service that provides wireguard or tailscale connection info to the resourcepoold, allowing the resource clients to
securely communicate with the resource.

# Terminology

resource.entities - entity - property


# Overview of deliverables

* http based daemon
  * using some http server crate
  * using yaml crate for configuration parsing
* command line tool
* unittests
* black box tests
* rust based client library

## stretch

* python based client library
* bash+curl based example

# Roadmap

## 0. kickoff - working HTTP server with dummy endpoints

Use hyper (library) + hyper_thungstenite (websockets) or rocket (fancy wrapper around a function per endpoint)

**As a ** client \
**I can do** http post to placeholder endpoints
* POST /lock?resource=name&description=whatIdo&priority=10
* GET / - return hello, eventually it can print an overview of available HW and use.

## 1. minimum viable product

**As a** devops engineer \
**I am able to** configure the service by changing datastructures in the main file.

The datastructure should be like
```
pool:
  resources:
    resource1:
      entities:
        programmer: {}
        dut: {}
```

**As a** client \
**I can do** a HTTP call using the command line tool, specifying the testbench by (unique) name. I get a resource busy
error or message that I have a lease. When the client cuts the connection the lock is released by the server.

DoD: automated test: service is run and client is run, running command, when a second client is started, it returns an error.

**As a** devops engineer \
**I am able to** get an overview of resource usage by sending HTTP GET /

## 2. Queueing

**As a** client
**I will have to** wait while the hardware is in use instead of seeing an error, and I will get a message when I obtain the lock.


DoD: automated test: service is run and client is run, when a second client is started, it waits until first is stopped and then runs the command.


## 3. Test parameterisation based on resource leased

**As a** devops engineer \
**I am able to** add metadata to the respo inventory, which can be retrieved by clients to eg. be able to retrieve the hostname of the testbench equipment.

**As a** devops engineer \
**I am able to** validate the respo inventory by running the tool with --validate-schema

**As a** devops engineer \
**I am able to** mutate the respo inventory by sending a HTTP POST with yaml data.

**As a** client
**I can** request a resoruce using a query composed of location and/or property sets.

example query: ``location=office1&attributes=[a]&entity_properties=[[c],[e]]``

example inventory:
```
pool:
  name: "software update testbenches"
  resources:
    resource1:
      attributes: ["a","b"]
      location: "office1"
      entities:
        programmer:
          attributes: ["c","d"]
          properties: # free form data user domain
            socket: 127.0.0.1:1234
            api_key: xyz
        dut:
          attributes: ["e","f"]
          properties:
            socket: 127.0.0.1:4567
            api_key: foobar
```

DoD: user story met and unittests for resource selection added.

## 4. persistence
**As a** devops engineer \
**I want** the respo inventory to be persisted to disk so that it survives restarts of the service.

