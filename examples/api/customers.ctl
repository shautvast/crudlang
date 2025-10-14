///gets the customer
fn get(id: u32) -> Customer:
    service.get(id)

fn post(customer: Customer):
    service.add(customer)
