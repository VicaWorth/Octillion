import mysql.connector  # Or your preferred MySQL library (e.g., PyMySQL)
import json
import sys, os

# Add the parent directory to the Python path
current_dir = os.path.dirname(os.path.abspath(__file__))
parent_dir = os.path.dirname(current_dir)
sys.path.append(parent_dir)

# Now you can import config
import config  # Assuming the file is named config.pys

def safe_int(value, default=0):
    """Safely converts a value to an integer.

    Args:
        value: The value to convert.
        default: The value to return if the conversion fails.

    Returns:
        The integer value, or the default value if conversion fails.
    """
    try:
        return int(value)
    except (ValueError, TypeError):
        return default

def safe_decimal(value, default=0.0):
    """Safely converts a value to a decimal.

    Args:
        value: The value to convert.
        default: The value to return if the conversion fails.

    Returns:
        The decimal value, or the default value if conversion fails.
    """
    try:
        return float(value)  # Use float for conversion to DECIMAL
    except (ValueError, TypeError):
        return default

def ensure_types(product_data):
    product_data['storeId'] = safe_int(product_data['storeId'])
    product_data['inventoryAvailable'] = safe_int(product_data['inventoryAvailable'])
    product_data['restrictedValue'] = safe_int(product_data['restrictedValue'])
    product_data['salesRank'] = safe_int(product_data['salesRank'])
    product_data['agreementId'] = safe_int(product_data['agreementId'])
    product_data['featuredProductId'] = safe_int(product_data['featuredProductId'])
    product_data['price'] = safe_decimal(product_data['price'])
    product_data['basePrice'] = safe_decimal(product_data['basePrice'])
    product_data['basePricePer'] = safe_decimal(product_data['basePricePer'])
    product_data['pricePer'] = safe_decimal(product_data['pricePer'])
    product_data['previousPurchaseQty'] = safe_int(product_data['previousPurchaseQty'])
    product_data['maxPurchaseQty'] = safe_int(product_data['maxPurchaseQty'])
    product_data['minWeight'] = safe_decimal(product_data['minWeight'])
    product_data['maxWeight'] = safe_decimal(product_data['maxWeight'])
    product_data['preparationTime'] = safe_int(product_data['preparationTime'])
    product_data['triggerQuantity'] = safe_int(product_data['triggerQuantity'])
    # product_data['dispItemSizeQty'] = safe_decimal(product_data['dispItemSizeQty'])
    # product_data['dispItemPackageQty'] = safe_int(product_data['dispItemPackageQty'])
    product_data['itemRetailSect'] = safe_int(product_data['itemRetailSect'])
    return product_data

def insert_product_data(product_data):
    """Inserts a single product's data into the Products table.

    Args:
        product_data: A dictionary containing the product data, matching the
            structure of the JSON example and the Products table schema.
    """
    try:
        # Establish a connection to the database
        conn = mysql.connector.connect(**config.db_config)
        cursor = conn.cursor()

        # Construct the SQL INSERT statement
        #  We'll use parameterized queries for security and to handle data type conversions.
        sql = """
            INSERT INTO Products (
                status, name, pid, upc, id, storeId, featured, inventoryAvailable,
                restrictedValue, salesRank, agreementId, featuredProductId, imageUrl,
                price, basePrice, basePricePer, pricePer, snapEligible, unitOfMeasure,
                sellByWeight, unitQuantity, displayUnitQuantityText, previousPurchaseQty,
                maxPurchaseQty, minWeight, maxWeight, isHhcProduct,
                prop65WarningIconRequired, isArProduct, isMtoProduct, isCustomizable,
                inStoreShoppingElig, preparationTime, isMarketplaceItem, triggerQuantity,
                idOfAisle, idOfShelf, idOfDepartment, warnings, requiresReturn,
                channelEligibility, productReview, dispItemSizeQty, dispItemPackageQty,
               dispUnitOfMeasure, itemRetailSect
            ) VALUES (
                %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s,
                %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s,
                %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s
            )
        """

        # Prepare the data for insertion
        #  Ensure the order matches the SQL statement, and handle nested JSON
        data = (
            product_data['status'],
            product_data['name'],
            product_data['pid'],
            product_data['upc'],
            product_data['id'],
            product_data['storeId'],
            product_data['featured'],
            product_data['inventoryAvailable'],
            product_data['restrictedValue'],
            product_data['salesRank'],
            product_data['agreementId'],
            product_data['featuredProductId'],
            product_data['imageUrl'],
            product_data['price'],
            product_data['basePrice'],
            product_data['basePricePer'],
            product_data['pricePer'],
            product_data['snapEligible'],
            product_data['unitOfMeasure'],
            product_data['sellByWeight'],
            product_data['unitQuantity'],
            product_data['displayUnitQuantityText'],
            product_data['previousPurchaseQty'],
            product_data['maxPurchaseQty'],
            product_data['minWeight'],
            product_data['maxWeight'],
            product_data['isHhcProduct'],
            product_data['prop65WarningIconRequired'],
            product_data['isArProduct'],
            product_data['isMtoProduct'],
            product_data['isCustomizable'],
            product_data['inStoreShoppingElig'],
            product_data['preparationTime'],
            product_data['isMarketplaceItem'],
            product_data['triggerQuantity'],
            product_data['idOfAisle'],
            product_data['idOfShelf'],
            product_data['idOfDepartment'],
            json.dumps(product_data.get('warnings', [])),  # Convert to JSON string, default to empty list
            product_data['requiresReturn'],
            json.dumps(product_data.get('channelEligibility', {})),  # Convert to JSON string, default to empty dict
            json.dumps(product_data.get('productReview', {})),  # Convert to JSON string, default to empty dict
            product_data['dispItemSizeQty'],
            product_data['dispItemPackageQty'],
            product_data['dispUnitOfMeasure'],
            product_data['itemRetailSect']
        )
        # Execute the SQL statement with the data
        cursor.execute(sql, data)

        # Commit the changes to the database
        conn.commit()
        print(f"Product {product_data['id']} inserted successfully.")

    except mysql.connector.Error as err:
        print(f"Error: {err}")
        # Consider more specific error handling (e.g., duplicate key errors)
        conn.rollback()  # Rollback changes in case of error

    finally:
        if conn and conn.is_connected():
            cursor.close()
            conn.close()

def valid_json(product_data):
    if 'id' in product_data:
        return True
    else:
        print("id not found in product_data")
        return False
    
def check_columns_exist(product_data):
    for key in ['dispItemSizeQty','dispItemPackageQty','dispUnitOfMeasure','itemRetailSect']:
        if key not in product_data:
            product_data[key] = None
    return product_data

def pull_from_json(directory):
    # Reads in all files from directory
    total_files = len(os.listdir(directory))
    print(f"Total files: {total_files}")
    for file in os.listdir(directory):
        with open(os.path.join(directory, file), 'r') as f:
            data = json.load(f)
            for product in data['primaryProducts']['response']['docs']:
                print(product)
                if not valid_json(product):
                    continue
                product = check_columns_exist(product)
                product_data = ensure_types(product)
                json.dump(product_data, sys.stdout, indent=4)
                insert_product_data(product_data)

if __name__ == "__main__":
    pull_from_json(sys.argv[1])  # Pass the directory path as a command-line argument