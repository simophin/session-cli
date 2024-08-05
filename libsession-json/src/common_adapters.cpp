//
// Created by Fanchao Liu on 28/6/2024.
//

#include "common_adapters.h"

#include <oxenc/base64.h>

void to_json(nlohmann::json &j, const Base64 &v)
{
    j = v.value.empty() ? "" : oxenc::to_base64(v.value);
}

void session::config::to_json(nlohmann::json &j, const profile_pic &v)
{
    if (v.url.empty())
    {
        j = nullptr;
    }
    else
    {
        j["url"] = v.url;
        j["key"] = oxenc::to_base64(v.key);
    }
}
