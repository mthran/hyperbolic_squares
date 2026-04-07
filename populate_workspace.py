"""Populate the workspace."""

import signac

project = signac.get_project()

job = project.open_job(
    {
        "n": 79,
        "final_size": 0.2,
        "replicate": 0,
    }
).init()

job = project.open_job(
    {
        "n": 87,
        "final_size": 0.2,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 95,
        "final_size": 0.2,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 315,
        "final_size": 0.1,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 346,
        "final_size": 0.1,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 376,
        "final_size": 0.1,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 1_257,
        "final_size": 0.05,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 1_383,
        "final_size": 0.05,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 1_508,
        "final_size": 0.05,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 491,
        "final_size": 0.08,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 540,
        "final_size": 0.08,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 590,
        "final_size": 0.08,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 7_855,
        "final_size": 0.02,
        "replicate": 0,
    }
).init()
job = project.open_job(
    {
        "n": 8_640,
        "final_size": 0.02,
        "replicate": 0,
    }
).init()

job = project.open_job(
    {
        "n": 9_425,
        "final_size": 0.02,
        "replicate": 0,
    }
).init()
