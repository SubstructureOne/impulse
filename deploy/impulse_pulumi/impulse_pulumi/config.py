import os

import pulumi


SSH_KEY_PATH = os.getenv("VULTR_SSH_KEY_PATH")
if SSH_KEY_PATH is None:
    raise ValueError("VULTR_SSH_KEY_PATH not set")


def prepend_env(
    env: dict[str, str | pulumi.Resource],
    command: str,
):
    """Takes an input like

    prepend_env({"VAR1": "value1", "VAR2": some_pulumi_output}, "bash mycommand.sh")

    And returns an output that will resolve to:

    '''VAR1="value1" VAR2="the_actual_output_value" bash mycommand.sh'''
    """
    return pulumi.Output.all(**env).apply(
        lambda kwargs: "".join(
            map(lambda item: f"""{item[0]}="{item[1]}" """, kwargs.items())
        )
        + command
    )
