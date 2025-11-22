# About

resourcepool(d)/respo(d) - a resource (eg network connected equipment test bench) leasing system

It's a Http service where you can request exclusive access to a resource, which is composed of a set of entities with
properties.
A testbench (bench) is typically used for hardware in the loop testing in software validation, and is composed of one or more DUT's, debugging hardware and measurement equipment.
Hence, a client would request access to a resource matching properties, and of which the entities match properties.

# Summary usage

As resource user, I want to do a HTTP POST /lock?location=myoffice&resource=myresource to request use of an a resource by name.
Upon receiving the message that the resource is locked by me, I should be able to use the returned resource (description)
until I cut the connection to the service. At that point the lock is considered implicitly released.

Clients get queued if the lock is not available, and can specify a priority value (to have a developer skip the queue over a CI/CD runner)

There can be language specific Client libraries (eg python) to use in test scenario's (eg pytest) that use the physical devices

There is a command line tool (rpclient) to claim maintenance on the test setup

The service maintains database/list with  office location, property tags, freeform dict of details (port numbers etc) persisted to disk.

Devops can do a HTTP POST /database with yaml to update the database (eg. from A CI/CD pipeline), which returns an HTTP error if it fails to validate.

# Far stretch ideas

For security each resource could be fitted with a local daemon that provides wireguard connection info to the resourcepoold,
allowing the resource users to securely use the resource.

# Terminology

resource.inventory - entity - property


# Overview of deliverables

* http based daemon
  * using some http server crate
  * using yaml crate for configuration parsing
* command line tool
* unittests
* black box tests
* rust based client library

## stretch

* GET / would show an overview of allocation of the resources
* python based client library
* bash+curl based example

# Roadmap

## 0. kickoff - working HTTP server with dummy endpoints

Use hyper (library) + hyper_thungstenite (websockets) or rocket (fancy wrapper around a function per endpoint)

**As a ** user \
**I can do** http post to placeholder endpoints
* POST /lock?resource=name&description=whatIdo&priority=10
* GET / - return hello, eventually it can print an overview of available HW and use.

## 1. minimum viable product

As a devops engineer \
I am able to configure the service by changing datastructures in the main file.

The datastructure should be like
```
pool:
  resources:
    resource1:
      inventory:
        programmer: {}
        dut: {}
```

**As a** developer \
**I can do** a HTTP call using the command line tool, specifying the testbench by (unique) name.

DoD: automated test: service is run and client is run, running command, when a second client is started, it returns an error.

## 2. Queueing

**As a** developer
**I will have to** wait while the hardware is in use instead of seeing an error, and I will get a message when I obtain the lock.


DoD: automated test: service is run and client is run, when a second client is started, it waits until first is stopped and then runs the command.


## 3. Test parameterisation based on testbench leased

**As a** devops engineer
**I am able to** add metadata to the respod database, which can be retrieved by clients to eg. be able to retrieve the hostname of the testbench equipment.

As a devops engineer
I am able to validate the respod database by running the tool with --validate-schema

As a devops engineer
I am able to mutate the respod database by sending a HTTP POST with yaml data.

As a developer
I am able to specify a location and tags that

```
pool:
  resources:
    resource1:
      properties: ["a","b"]
      location: "office1"
      inventory:
        programmer:
          properties: ["c","d"]
        dut:
          properties: ["e","f"]
```

DoD: user story met and unittests for selection added.

## 4. persistence
As a devops engineer
I want the respod database to be persisted to disk so that it survives restarts of the service.

