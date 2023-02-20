use crate::address::Address;

struct ChannelsRequest {
    account: Address,
    destination_account: Option<Address>,

}



struct ChannelPaginator {

}

impl Paginator for ChannelPaginator {}