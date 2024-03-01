import sys

sys.path.append('../')
from near_multinode_rpc_provider import MultiNodeJsonProviderError, MultiNodeJsonProvider
import json
import time
# from redis_provider import list_top_pools, list_token_price, list_token_metadata
# from utils import combine_pools_info
from decimal import *
# import requests
from contract_handler import RpcHandler
import globals
from config import GlobalConfig
import random
import os
import requests

global_config = GlobalConfig()
# Disable buffering of standard output streams
sys.stdout = os.fdopen(sys.stdout.fileno(), 'w', 1)
# Disable buffering of standard error streams
sys.stderr = os.fdopen(sys.stderr.fileno(), 'w', 1)


def handle_buy_buck_one(random_num):
    try:
        conn = MultiNodeJsonProvider()
        ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
        b = "".join([chr(x) for x in ret["result"]])
        amount_in = int(json.loads(b))
        print("first fund_amount:", amount_in)
        if amount_in > 0:
            handle_flow(amount_in, random_num)
            print("Wait for 60 seconds for the second verification")
            time.sleep(60)
            ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
            b = "".join([chr(x) for x in ret["result"]])
            amount_in = int(json.loads(b))
            print("second fund_amount:", amount_in)
            if amount_in > 0:
                handle_buy_buck_two(600)
            else:
                print("second not fund_amount")
        else:
            print("first not fund_amount")
    except MultiNodeJsonProviderError as e:
        print("RPC Error: ", e)
    except Exception as e:
        print("Error: ", e)
        print("Wait for 60 seconds for the second verification")
        time.sleep(60)
        conn = MultiNodeJsonProvider()
        ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
        b = "".join([chr(x) for x in ret["result"]])
        amount_in = int(json.loads(b))
        print("second fund_amount:", amount_in)
        if amount_in > 0:
            handle_buy_buck_two(600)
        else:
            print("second not fund_amount")


def handle_buy_buck_two(random_num):
    try:
        conn = MultiNodeJsonProvider()
        ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
        b = "".join([chr(x) for x in ret["result"]])
        amount_in = int(json.loads(b))
        print("retry fund_amount:", amount_in)
        if amount_in > 0:
            handle_flow(amount_in, random_num)
            print("Wait for 60 seconds for the second verification")
            time.sleep(60)
            ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
            b = "".join([chr(x) for x in ret["result"]])
            amount_in = int(json.loads(b))
            print("retry fund_amount:", amount_in)
            if amount_in > 0:
                handle_buy_buck_two(600)
            else:
                print("retry verification not fund_amount")
        else:
            print("retry not fund_amount")
    except MultiNodeJsonProviderError as e:
        print("RPC Error: ", e)
    except Exception as e:
        print("Error: ", e)
        print("Wait for 60 seconds for the second verification")
        time.sleep(60)
        conn = MultiNodeJsonProvider()
        ret = conn.view_call(global_config.buyback_contract, "get_available_fund_amount", b'')
        b = "".join([chr(x) for x in ret["result"]])
        amount_in = int(json.loads(b))
        print("retry fund_amount:", amount_in)
        if amount_in > 0:
            handle_buy_buck_two(600)
        else:
            print("retry verification not fund_amount")


def handle_flow(amount_in, random_num):
    print("random max num:", random_num)
    num = random.randint(1, random_num)
    print("random num:", num)
    time.sleep(num)

    query_list_pools_url = global_config.indexer_url
    requests.packages.urllib3.disable_warnings()
    list_pools_data_ret = requests.get(url=query_list_pools_url, verify=False)
    pools = json.loads(list_pools_data_ret.text)

    actions = []
    buyback_pool_one = {}
    buyback_pool_two = {}
    for pool in pools:
        if pool["id"] == global_config.buyback_pool_one:
            buyback_pool_one = pool
        if pool["id"] == global_config.buyback_pool_two:
            buyback_pool_two = pool

    if buyback_pool_one != {} and buyback_pool_two != {}:
        one_account_ids = buyback_pool_one["token_account_ids"]
        one_amounts = buyback_pool_one["amounts"]
        if one_account_ids[0] == global_config.buyback_token_in_contract:
            one_in_balance = one_amounts[0]
            one_out_balance = one_amounts[1]
            one_token_in = one_account_ids[0]
            one_token_out = one_account_ids[1]
        else:
            one_in_balance = one_amounts[1]
            one_out_balance = one_amounts[0]
            one_token_in = one_account_ids[1]
            one_token_out = one_account_ids[0]
        one_amount_out = get_token_flow_ratio(amount_in, one_in_balance, one_out_balance, buyback_pool_one["total_fee"])
        print("one_amount_out:", one_amount_out)
        one_min_amount_out = int(decimal_mult(one_amount_out, 0.997))
        action_one = {
            "pool_id": int(buyback_pool_one["id"]),
            "token_in": one_token_in,
            "amount_in": str(amount_in),
            "token_out": one_token_out,
            "min_amount_out": str(one_min_amount_out)
        }
        actions.append(action_one)
        two_account_ids = buyback_pool_two["token_account_ids"]
        two_amounts = buyback_pool_two["amounts"]
        if two_account_ids[1] == global_config.buyback_token_out_contract:
            two_in_balance = two_amounts[0]
            two_out_balance = two_amounts[1]
            two_token_in = two_account_ids[0]
            two_token_out = two_account_ids[1]
        else:
            two_in_balance = two_amounts[1]
            two_out_balance = two_amounts[0]
            two_token_in = two_account_ids[1]
            two_token_out = two_account_ids[0]
        two_amount_out = get_token_flow_ratio(one_amount_out, two_in_balance, two_out_balance,
                                              buyback_pool_two["total_fee"])
        print("two_amount_out:", two_amount_out)
        two_min_amount_out = int(decimal_mult(two_amount_out, 0.997))
        action_two = {
            "pool_id": int(buyback_pool_two["id"]),
            "token_in": two_token_in,
            "amount_in": None,
            "token_out": two_token_out,
            "min_amount_out": str(two_min_amount_out)
        }
        actions.append(action_two)
    print("actions:", actions)
    signer = globals.get_signer_account(global_config.signer_account_id)
    burrow_handler = RpcHandler(signer, global_config.buyback_contract)
    ret = burrow_handler.do_buyback(actions)
    # print("buyback:", ret)
    return ret


def get_token_flow_ratio(token_in_amount, token_in_balance, token_out_balance, fee):
    try:
        token_in_amount = int(token_in_amount)
        token_in_balance = int(token_in_balance)
        token_out_balance = int(token_out_balance)
        fee = int(fee)
        ratio = token_in_amount * (10000 - fee) * token_out_balance / (
                10000 * token_in_balance + token_in_amount * (10000 - fee))
        return int(ratio)
    except Exception as e:
        print("get ratio error:", e)
        return 0


def format_decimal_float(number):
    format_number = "{0:.16f}".format(Decimal(number))
    if '.' in format_number:
        return float(format_number[:format_number.index('.') + 1 + 8])
    return float(format_number)


def decimal_mult(number_one, number_two):
    return Decimal(str(number_one)) * Decimal(str(number_two))


if __name__ == "__main__":

    start_time = int(time.time())
    print("Staring buy back ...")
    handle_buy_buck_one(3600)
    end_time = int(time.time())
    print("buy back end")
    print("buy back consuming time:{}", start_time - end_time)

