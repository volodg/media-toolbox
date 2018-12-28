#!/bin/bash
diesel migration run --database-url='postgres://postgres:docker@localhost:5432/peers_test'
