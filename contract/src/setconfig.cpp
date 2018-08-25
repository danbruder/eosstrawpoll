#include <eosstrawpoll/contract.hpp>

namespace eosstrawpoll
{

void contract::setconfig(
    const uint16_t max_new_polls,
    const uint16_t max_popular_polls,
    const uint16_t max_new_donations,
    const uint16_t max_title_len,
    const uint16_t max_options_len,
    const uint16_t max_option_len,
    const uint16_t max_account_list_len,
    const uint16_t max_writein_len,
    const double popularity_gravity,
    const uint16_t max_metadata_len,
    const uint64_t profile_unlock_threshold,
    const string &metadata)
{
    require_auth(_self);

    _config = config{
        .max_new_polls = max_new_polls,
        .max_popular_polls = max_popular_polls,
        .max_new_donations = max_new_donations,
        .max_title_len = max_title_len,
        .max_options_len = max_options_len,
        .max_option_len = max_option_len,
        .max_account_list_len = max_account_list_len,
        .max_writein_len = max_writein_len,
        .popularity_gravity = popularity_gravity,
        .max_metadata_len = max_metadata_len,
        .profile_unlock_threshold = profile_unlock_threshold,
        .metadata = metadata};
    _configs.set(_config, _self);

    prune_new_donations();
    prune_new_polls();
    prune_popular_polls();
};
} // namespace eosstrawpoll